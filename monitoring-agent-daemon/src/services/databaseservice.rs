use monitoring_agent_lib::proc::{ProcsLoadavg, ProcsMeminfo};
use r2d2::Pool;
use r2d2_mysql::mysql::params;
use r2d2_mysql::mysql::prelude::Queryable;
use r2d2_mysql::mysql::OptsBuilder;
use r2d2_mysql::mysql::Row;
use r2d2_mysql::mysql::TxOpts;
use r2d2_mysql::MySqlConnectionManager;
use bb8_postgres::tokio_postgres::tls::NoTls;
use bb8_postgres::tokio_postgres::Config;
use bb8_postgres::PostgresConnectionManager;
use rust_decimal::Decimal;

use crate::common::configuration::DatabaseConfig;
use crate::common::configuration::DatabaseType;
use crate::common::Status;
use crate::common::ApplicationError;

/**
 * Database Service.
 * 
 * This enum represents the database service. It is used to interact with the database.
 * 
 */
#[derive(Debug)]
pub enum DbService {
    MariaDb(MariaDbService),
    PostgresDb(PostgresDbService),
}

impl DbService {

    /**
     * Create a new database service.
     * 
     * `database_config`: The database configuration.
     * `server_name`: The server name.
     * 
     * Returns: A new database service.
     * 
     * Errors:
     * - If there is an error creating the database service.
     * 
     */
    pub async fn new(database_config: &DatabaseConfig, server_name: &str) -> Result<DbService, ApplicationError> {
        match &database_config.dbtype {
            DatabaseType::Maria => Ok(DbService::MariaDb(MariaDbService::new(database_config, server_name)?)),
            DatabaseType::Mysql => Ok(DbService::MariaDb(MariaDbService::new(database_config, server_name)?)),
            DatabaseType::Postgres => Ok(DbService::PostgresDb(PostgresDbService::new(database_config, server_name).await?)),
        }
    }

    /**
     * Insert a monitor status into the database.
     * 
     * `name`: The name of the monitor.
     * `status`: The status of the monitor.
     * 
     * Returns: Ok if the status was inserted successfully.
     * 
     * Errors:
     * - If there is an error inserting the status.
     * - If there is an error starting a transaction.
     * - If there is an error committing the transaction.
     * 
     */
    pub async fn insert_monitor_status(&self, name: &str, status: &Status) -> Result<(), ApplicationError> {
        match self {
            DbService::MariaDb(service) => service.insert_monitor_status(name, status),
            DbService::PostgresDb(service) => service.insert_monitor_status(name, status).await,
        }
    }

    /**
     * Store the load average in the database.
     * 
     * `loadavg`: The load average to store.
     * 
     * Returns: Ok if the load average was stored successfully.
     * 
     * Errors:
     * - If there is an error storing the load average.
     * - If there is an error starting a transaction.
     * 
     */
    pub async fn store_loadavg(&self, loadavg: &ProcsLoadavg) -> Result<(), ApplicationError> {
        match self {
            DbService::MariaDb(service) => service.store_loadavg(loadavg),
            DbService::PostgresDb(service) => service.store_loadavg(loadavg).await,
        }
    }

    /**
     * Store the meminfo in the database.
     * 
     * `meminfo`: The meminfo to store.
     * 
     * Returns: Ok if the meminfo was stored successfully.
     * 
     * Errors:
     * - If there is an error storing the meminfo.
     * - If there is an error starting a transaction.
     * 
     */
    pub async fn store_meminfo(&self, meminfo: &ProcsMeminfo) -> Result<(), ApplicationError> {
        match self {
            DbService::MariaDb(service) => service.store_meminfo(meminfo),
            DbService::PostgresDb(service) => service.store_meminfo(meminfo).await,
        }
    }

    /**
     * Get the database representation of the status.
     * 
     * `status`: The status to get the database representation of.
     * 
     * Returns: The database representation.
     * 
     */
    fn get_status_db_repr(status: &Status) -> String {
        match &status {
            Status::Error { message: _ } => "ERROR".to_string(),
            Status::Ok => "OK".to_string(),
            Status::Unknown => "UNKNOWN".to_string(),
        }
    }

    /**
     * Get the message from the status.
     *
     * `status`: The status to get the message from.
     *
     * Returns: The message.
     *
     */
    fn get_message(status: &Status) -> Option<String> {
        match status {
            Status::Error { message } => Some(message.clone()),
            _ => None,
        }
    }

    /**
     * Query long running queries.
     * 
     * `max_query_time`: The maximum query time.
     * 
     * Returns: The long running queries.
     * 
     * Errors:
     * - If there is an error querying the long running queries.
     * - If there is an error starting a transaction.
     * - If there is an error committing the transaction.
     * 
     */
    pub async fn query_long_running_queries(&self, max_query_time: u32) -> Result<Vec<String>, ApplicationError> {
        match self {
            DbService::MariaDb(service) => service.query_long_running_queries(max_query_time),
            DbService::PostgresDb(service) => service.query_long_running_queries(max_query_time).await,
        }
    }
}

/**
 * `MariaDB` Service.
 * 
 * This struct represents a `MariaDB` service. It is used to interact with the `MariaDB` database.
 * 
 */
#[derive(Debug)]
pub struct MariaDbService {
    /// The database connection pool.
    pool: Pool<MySqlConnectionManager>,
    /// Server name
    server_name: String
}

impl MariaDbService {
    /**
     * Create a new `MariaDB` service.
     * 
     * `database_config`: The database configuration.
     * 
     * Returns: A new `MariaDB` service.
     * 
     * Errors:
     * - If there is an error creating the pool.
     */
    pub fn new(database_config: &DatabaseConfig, server_name: &str) -> Result<MariaDbService, ApplicationError> {

        let manager = r2d2_mysql::MySqlConnectionManager::new(OptsBuilder::new()
            .ip_or_hostname(Some(database_config.host.clone()))
            .db_name(Some(database_config.db_name.clone()))
            .user(Some(database_config.user.clone()))
            .pass(Some(database_config.password.clone()))
            .tcp_port(database_config.port)
            .init(vec![
                "SET time_zone = '+00:00';",
            ]));
        let pool = r2d2::Pool::builder()
            .max_size(database_config.max_connections)
            .min_idle(Some(database_config.min_connections))
            .build(manager)
            .map_err(|err| ApplicationError::new(&err.to_string()))?;
        /*
         * Verify connection
         */
        Ok(MariaDbService {
            pool,
            server_name: server_name.to_string()
        })
    }

    /**
     * Insert a monitor status into the database.
     * 
     * `name`: The name of the monitor.
     * `status`: The status of the monitor.
     * `message`: The message associated with the status.
     * 
     * Returns: Ok if the status was inserted successfully.
     * 
     * Errors:
     * - If there is an error inserting the status.
     * - If there is an error starting a transaction.
     * - If there is an error committing the transaction.
     * 
     */
    pub fn insert_monitor_status(&self, name: &str, status: &Status) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get().map_err(|err| ApplicationError::new(&err.to_string()))?;
        let mut tx = conn.start_transaction(TxOpts::default()).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.exec_drop("INSERT INTO monitor_status (server_name, monitor_name, status, log_time, message) VALUES (:server_name,:name, :status, now(3), :message)", params! {
            "server_name" => self.server_name.to_string(),
            "name" => &name,
            "status" => DbService::get_status_db_repr(status),
            "message" => DbService::get_message(status),
        }).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.commit().map_err(|err| ApplicationError::new(&err.to_string()))?;
        Ok(())
    }

    /**
     * Store the load average in the database.
     * 
     * `loadavg`: The load average to store.
     * 
     * Returns: Ok if the load average was stored successfully.
     * 
     * Errors:
     * - If there is an error storing the load average.
     * - If there is an error starting a transaction.
     */
    pub fn store_loadavg(&self, loadavg: &ProcsLoadavg) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get().map_err(|err| ApplicationError::new(&err.to_string()))?;
        let mut tx = conn.start_transaction(TxOpts::default()).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.exec_drop("INSERT INTO loadavg (server_name, loadavg1min, loadavg5min, loadavg10min, num_processes, num_running_processes, log_time) VALUES (:server_name, :loadavg1min, :loadavg5min, :loadavg10min, :num_processes, :num_running_processes, now(3))", params! {
            "server_name" => self.server_name.to_string(),
            "loadavg1min" => loadavg.loadavg1min,
            "loadavg5min" => loadavg.loadavg5min,
            "loadavg10min" => loadavg.loadavg10min,
            "num_processes" => loadavg.total_number_of_processes,
            "num_running_processes" => loadavg.current_running_processes,
        }).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.commit().map_err(|err| ApplicationError::new(&err.to_string()))?;
        Ok(())
    }
    /**
     * Store the meminfo in the database.
     * 
     * `meminfo`: The meminfo to store.
     * 
     * Returns: Ok if the meminfo was stored successfully.
     * 
     * Errors:
     * - If there is an error storing the meminfo.
     * - If there is an error starting a transaction.
     */
    pub fn store_meminfo(&self, meminfo: &ProcsMeminfo) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get().map_err(|err| ApplicationError::new(&err.to_string()))?;
        let mut tx = conn.start_transaction(TxOpts::default()).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.exec_drop("INSERT INTO meminfo (server_name, freemem, mem_percent_used, freeswap, swap_percent_used, log_time) VALUES (:server_name, :freemem, :mem_percent_used, :freeswap, :swap_percent_used, now(3))", params! {
            "server_name" => self.server_name.to_string(),
            "freemem" => meminfo.memfree,
            "mem_percent_used" => ProcsMeminfo::get_percent_used(meminfo.memfree, meminfo.memtotal),
            "freeswap" => meminfo.swapfree,
            "swap_percent_used" => ProcsMeminfo::get_percent_used(meminfo.swapfree, meminfo.swaptotal),
        }).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.commit().map_err(|err| ApplicationError::new(&err.to_string()))?;       
        Ok(())
    }

    /**
     * Query long running queries.
     * 
     * `max_query_time`: The maximum query time.
     * 
     * Returns: The long running queries.
     * 
     * Errors:
     * - If there is an error querying the long running queries.
     * - If there is an error starting a transaction.
     * - If there is an error committing the transaction.
     * 
     */
    pub fn query_long_running_queries(&self, max_query_time: u32) -> Result<Vec<String>, ApplicationError> {
        let mut conn = self.pool.get().map_err(|err| ApplicationError::new(&err.to_string()))?;
        let mut tx = conn.start_transaction(TxOpts::default()).map_err(|err| ApplicationError::new(&err.to_string()))?;
        let params = params! {
            "max_query_time" => max_query_time,
        };
        let result = tx.exec_map("SELECT * FROM INFORMATION_SCHEMA.PROCESSLIST WHERE COMMAND != 'Sleep' AND TIME > :max_query_time", params, |row: Row| {
            let id: u32 = row.get(0).unwrap_or(0);
            let info: String = row.get(7).unwrap_or("unknown".to_string());
            format!("id: {id}, info: {info}")
        }).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.commit().map_err(|err| ApplicationError::new(&err.to_string()))?;
        Ok(result)
    }

}


#[derive(Debug)]
pub struct PostgresDbService {
    /// The database connection pool.
    pool: bb8::Pool<PostgresConnectionManager<NoTls>>,
    /// Server name
    server_name: String
}

impl PostgresDbService {
    
    /**
     * Create a new `Postgres` service.
     * 
     * `database_config`: The database configuration.
     * 
     * Returns: A new `Postgres` service.
     * 
     * Errors:
     * - If there is an error creating the pool.
     */
    pub async fn new(database_config: &DatabaseConfig, server_name: &str) -> Result<PostgresDbService, ApplicationError> {

        let manager = bb8_postgres::PostgresConnectionManager::new(Config::new()
            .host(&database_config.host)
            .dbname(&database_config.db_name)
            .user(&database_config.user)
            .password(&database_config.password)
            .port(database_config.port).clone(), NoTls);

        let pool = bb8::Pool::builder()
            .max_size(database_config.max_connections)
            .min_idle(Some(database_config.min_connections))
            .build(manager)
            .await
            .map_err(|err| ApplicationError::new(&err.to_string()))?;
        /*
         * Verify connection
         */
        Ok(PostgresDbService {
            pool,
            server_name: server_name.to_string()
        })
    }

    /**
     * Insert a monitor status into the database.
     * 
     * `name`: The name of the monitor.
     * `status`: The status of the monitor.
     * `message`: The message associated with the status.
     * 
     * Returns: Ok if the status was inserted successfully.
     * 
     * Errors:
     * - If there is an error inserting the status.
     * - If there is an error starting a transaction.
     * - If there is an error committing the transaction.
     * 
     */
    pub async fn insert_monitor_status(&self, name: &str, status: &Status) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        let tx = conn.transaction().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.execute("INSERT INTO monitor_status (id, server_name, monitor_name, status, log_time, message) VALUES (nextval('seq_monitor_status'), $1, $2, $3, now(), $4)", &[
            &self.server_name,
            &name,
            &DbService::get_status_db_repr(status),
            &DbService::get_message(status),
        ]).await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.commit().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        Ok(())
    }

    /**
     * Store the load average in the database.
     * 
     * `loadavg`: The load average to store.
     * 
     * Returns: Ok if the load average was stored successfully.
     * 
     * Errors:
     * - If there is an error storing the load average.
     * - If there is an error starting a transaction.
     */
    pub async fn store_loadavg(&self, loadavg: &ProcsLoadavg) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        let tx = conn.transaction().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.execute("INSERT INTO loadavg (id, server_name, loadavg1min, loadavg5min, loadavg10min, num_processes, num_running_processes, log_time) VALUES (nextval('seq_loadavg'), $1, $2, $3, $4, $5, $6, now())", &[
            &self.server_name,
            &loadavg.loadavg1min.map(|f|Decimal::try_from(f).ok()),
            &loadavg.loadavg5min.map(|f|Decimal::try_from(f).ok()),
            &loadavg.loadavg10min.map(|f|Decimal::try_from(f).ok()),
            &loadavg.total_number_of_processes.map(i64::from),
            &loadavg.current_running_processes.map(i64::from),
        ]).await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.commit().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        Ok(())
    }

    /**
     * Store the meminfo in the database.
     * 
     * `meminfo`: The meminfo to store.
     * 
     * Returns: Ok if the meminfo was stored successfully.
     * 
     * Errors:
     * - If there is an error storing the meminfo.
     * - If there is an error starting a transaction.
     */
    pub async fn store_meminfo(&self, meminfo: &ProcsMeminfo) -> Result<(), ApplicationError> {
        let mut conn = self.pool.get().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        let tx = conn.transaction().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.execute("INSERT INTO meminfo (id, server_name, freemem, mem_percent_used, freeswap, swap_percent_used, log_time) VALUES (nextval('seq_meminfo'), $1, $2, $3, $4, $5, now())", &[
            &self.server_name,
            &meminfo.memfree.map(|x| i32::try_from(x).ok()),
            &ProcsMeminfo::get_percent_used(meminfo.memfree, meminfo.memtotal).map(|f|Decimal::try_from(f).ok()),
            &meminfo.swapfree.map(|x| i32::try_from(x).ok()),
            &ProcsMeminfo::get_percent_used(meminfo.swapfree, meminfo.swaptotal).map(|f|Decimal::try_from(f).ok()),
        ]).await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.commit().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        Ok(())
    }

    /**
     * Query long running queries.
     * 
     * `max_query_time`: The maximum query time.
     * 
     * Returns: The long running queries.
     * 
     */
    pub async fn query_long_running_queries(&self, max_query_time: u32) -> Result<Vec<String>, ApplicationError> {
        let mut conn = self.pool.get().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        let tx = conn.transaction().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        let result = tx.query("SELECT * FROM pg_stat_activity WHERE state = 'active' AND now() - query_start > interval '1 second' * $1", &[&f64::from(max_query_time)]).await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        let queries = PostgresDbService::map_result(result);
        tx.commit().await.map_err(|err| ApplicationError::new(&err.to_string()))?;
        Ok(queries)
    }

    /**
     * Map the result.
     * 
     * `result`: The result to map.
     * 
     * Returns: The mapped result.
     * 
     * Errors:
     * - If there is an error mapping the result.
     */
    fn map_result(result: Vec<r2d2_postgres::postgres::Row>) -> Vec<String> {
        let mut queries: Vec<String> = Vec::new();
        for row in result {
            let id: u32 = row.get(0);
            let client: String = row.get(6);
            queries.push(format!("id: {id}, client: {client}"));
        }
        queries
    }
}
