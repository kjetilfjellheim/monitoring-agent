use monitoring_agent_lib::proc::{ProcsLoadavg, ProcsMeminfo};
use mysql::{self, OptsBuilder, Pool, PoolConstraints, PoolOpts, TxOpts }; // Import the `query` function and the `exec` function
use mysql::params;
use mysql::prelude::Queryable;

use crate::common::configuration::DatabaseConfig;
use crate::common::Status;
use crate::common::ApplicationError;

/**
 * `MariaDB` Service.
 * 
 * This struct represents a `MariaDB` service. It is used to interact with the `MariaDB` database.
 * 
 */
#[derive(Debug)]
pub struct MariaDbService {
    /// The database connection pool.
    pool: Pool,
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
    pub fn new(database_config: &DatabaseConfig) -> Result<MariaDbService, ApplicationError> {
        /*
         * Create pool
         */
         let conn_opts = OptsBuilder::new()
            .ip_or_hostname(Some(&database_config.host))
            .db_name(Some(&database_config.db_name))
            .user(Some(database_config.user.as_str()))
            .pass(Some(database_config.password.as_str()))
            .tcp_port(database_config.port)        
            .pool_opts(PoolOpts::default().with_constraints(PoolConstraints::new(database_config.min_connections, database_config.max_connections).unwrap()).with_reset_connection(false).with_check_health(false))            
            ;

        let pool = mysql::Pool::new(conn_opts).map_err(|err| ApplicationError::new(&err.to_string()))?;
        /*
         * Verify connection
         */
        Ok(MariaDbService {
            pool,
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
        let mut tx = self.pool.start_transaction(TxOpts::default()).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.exec_drop("INSERT INTO monitor_status (monitor_name, status, log_time, message) VALUES (:name, :status, now(3), :message)", params! {
            "name" => &name,
            "status" => MariaDbService::get_status_db_repr(status),
            "message" => MariaDbService::get_message(status),
        }).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.commit().map_err(|err| ApplicationError::new(&err.to_string()))?;
        Ok(())
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
        let mut tx = self.pool.start_transaction(TxOpts::default()).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.exec_drop("INSERT INTO loadavg (loadavg1min, loadavg5min, loadavg10min, num_processes, num_running_processes, log_time) VALUES (:loadavg1min, :loadavg5min, :loadavg10min, :num_processes, :num_running_processes, now(3))", params! {
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
        let mut tx = self.pool.start_transaction(TxOpts::default()).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.exec_drop("INSERT INTO meminfo (freemem, mem_percent_used, freeswap, swap_percent_used, log_time) VALUES (:freemem, :mem_percent_used, :freeswap, :swap_percent_used, now(3))", params! {
            "freemem" => meminfo.memfree,
            "mem_percent_used" => ProcsMeminfo::get_percent_used(meminfo.memfree, meminfo.memtotal),
            "freeswap" => meminfo.swapfree,
            "swap_percent_used" => ProcsMeminfo::get_percent_used(meminfo.memfree, meminfo.memtotal),
        }).map_err(|err| ApplicationError::new(&err.to_string()))?;
        tx.commit().map_err(|err| ApplicationError::new(&err.to_string()))?;       
        Ok(())
    }

}