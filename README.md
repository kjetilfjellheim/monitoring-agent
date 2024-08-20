# Monitoring agent

## Goals

The goals for the code is to develop a simple monitoring agent with the following functionality.
- Open tcp connection.
- Http request
- Commands
- Database query check
- Load average
- Memory consumption
- Systemd service
- Process monitoring

## Development
The codebase is most for learning more advanced rust code while trying to create something I can use in my home network.

## Testing
Run `cargo test`
Tests now includes all code except the tls as I have not found a way to add tls server to the tests or to the docker. It is currently ignored.

## Build

Install rust
Run `cargo build`

## Use

### Run

Run as non daemon `./monitoring-agent --config ./config.json `
Run as daemon `./monitoring-agent --daemon`

### Arguments
| Argument  | Description | Default | 
| ------------- | ------------- | ------------- |
| config | Configuration file | /etc/monitoring-agent/config.json | 
| logfile | File logger  | /var/log/monitoring-agent-daemon/monitoring-agent.log | 
| daemon | Run application as a daemon  | false | 
| stdout_errorlevel | Level for stdout. Valid values are TRACE, DEBUG, INFO, WARN, ERROR. | ERROR |
| file_errorlevel | Level for file. Valid values are TRACE, DEBUG, INFO, WARN, ERROR. | ERROR |
| test | Test a configuration file | false | 
| pidfile | Location of the pid file. Only in daemon mode. | /tmp/monitoring-agent.pid |

### Server configuration

| Config  | Description | 
| ------------- | ------------- |
| Name | Name of the server. This is used when storing the monitor information. |
| server.ip | Ip4 address | 
| server.port | Port | 

#### Tcp monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| details.type | Type of monitor. Must be tcp | 
| details.host | Host/ip to connect to. | 
| details.port | Port to connect to. | 

#### Http monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| details.type | Type of monitor. Must be http | 
| details.url | Url to make the request to. | 
| details.method | Method like post, put, delete, get, option, head | 
| details.body | Body to send | 
| details.headers | Headers to send | 
| details.useBuiltinRootCerts | Use systems built in root certificates |
| details.acceptInvalidCerts | Should the system accept invalid certificates |
| details.tlsInfo | Hyper extension carrying extra TLS layer information |
| details.rootCertificate | Import a root server certificate. Must be a pem file |
| details.identity | Import client identity. Must be a pem file |
| details.identityPassword | Client identity password |

#### Command monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| details.type | Type of monitor. Must be command | 
| details.command | Command to run | 
| details.args | List of command arguments | 
| details.expected | Expected response | 

#### LoadAvg monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| details.type | Type of monitor. Must be loadAvg | 
| details.threshold1min | Threshold value for 1 minute average | 
| details.threshold5min | Threshold value for 5 minute average | 
| details.threshold15min | Threshold value for 15 minute average | 
| details.storeValues | Store values in the database if configured. | 

#### Mem monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| details.type | Type of monitor. Must be mem | 
| details.maxPercentageMemUsed | Max percentage of memory used | 
| details.maxPercentageSwapUsed | Max percentage of swap used | 
| details.storeValues | Store values in the database if configured. | 

#### Systemctl monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| details.type | Type of monitor. Must be mem | 
| details.active | Array of systemd services which must be active | 

#### Database monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| details.type | Type of monitor. Must be mem | 
| details.config | Optional. If not given the general database config must be given |
| details.config.type | Type of database. Supported are Postgres, Mysql and Maria |
| details.config.host | Database server host name |
| details.config.port | Database server host port |
| details.config.database | Database name |
| details.config.user | Username |
| details.config.password | Password |
| details.config.minConnections | Connection pool minimum connections |
| details.config.maxConnections | Connection pool maximum connections |
| details.maxQueryTime | Max time for a query to take | 

#### Process monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| details.type | Type of monitor. Must be process | 
| details.config | Optional. If not given the general database config must be given |
| details.config.type | Type of database. Supported are Postgres, Mysql and Maria |
| details.config.applicationNames | Array of application names to monitor |
| details.config.maxMemUsage | Max memory use befor monitor changes to error |
| details.config.storeValues | Store values from the monitor in the statm table |

#### Example file

```
{
    "database": {
        "type": "Maria",
        "host": "",
        "port": 3306,
        "user": "monitor-agent-rw",
        "password": "",
        "database": "monitor-agent",
        "minConnections": 10,
        "maxConnections": 100
    },
    "server": {
            "name": "dev",
            "ip": "127.0.0.1",
            "port":64999
    },
    "monitors":[
        {
            "name":"Systemctl ssh",
            "schedule": "0 */1 * * * *",
            "details": {
                "type": "systemctl",
                "active": ["ssh"],
                "storeValues": true
            }
        },
        {
            "name":"Apache TCP",
            "schedule": "0 */1 * * * *",
            "details": {
                "type": "tcp",
                "host": "127.0.0.1",
                "port": 443
            }
        },
        {
            "name":"MariaDB TCP",
            "schedule": "0 */1 * * * *",
            "details": {
                "type": "tcp",
                "host": "127.0.0.1",
                "port": 3306
            }
        },
        {
            "name":"PostgresDB TCP",
            "schedule": "0 */1 * * * *",
            "details": {
                "type": "tcp",
                "host": "127.0.0.1",
                "port": 5432
            }
        },
        {
            "name":"Loadavg",
            "schedule": "0 */1 * * * *",
            "details": {
                "type": "loadAvg",
                "threshold1min": 2.0,
                "threshold5min": 2.0,
                "threshold15min": 2.0,
                "storeValues": true
            }
        },
        {
            "name":"Mem",
            "schedule": "0 */1 * * * *",
            "details": {
                "type": "mem",
                "maxPercentageMemUsed": 75.0,
                "maxPercentageSwapUsed": 75.0,
                "storeValues": true
            }
        }
    ]
}

```

## Setup as systemd service

### Important
Type=simple This is important as the application fails otherwise.

### Setup
Add the monitoring-agent command to /usr/local/bin
Add configuration file to /etc/monitoring-agent/config.json

Create a file in /etc/systemd/system/ called monitoring-agent.service
```
[Unit]
Description=Monitoring agent
DefaultDependencies=no
Before=shutdown.target

[Service]
ExecStart=/usr/local/bin/monitoring-agent
Type=simple
Restart=on-failure
TimeoutStartSec=10

[Install]
WantedBy=default.target
```

Run the command : systemctl daemon-reload
Run the command : systemctl enable monitoring-agent.service
Run the command : systemctl start monitoring-agent.service
Run the command : systemctl status monitoring-agent.service 
You should see the following result.
```
● monitoring-agent.service - Monitoring agent
     Loaded: loaded (/etc/systemd/system/monitoring-agent.service; enabled; preset: enabled)
     Active: active (running) since Sun 2024-07-21 01:52:36 CEST; 18h ago
    Process: 234467 ExecStart=/usr/local/bin/monitoring-agent --daemon (code=exited, status=0/SUCCESS)
   Main PID: 234470 (monitoring_agen)
      Tasks: 17 (limit: 18118)
     Memory: 15.6M (peak: 17.5M)
        CPU: 7min 19.565s
     CGroup: /system.slice/monitoring-agent.service
             └─234470 /usr/local/bin/monitoring-agent --daemon

juli 21 01:52:36 alpha-legion systemd[1]: Starting monitoring-agent.service - Monitoring agent...
juli 21 01:52:36 alpha-legion monitoring-agent[234467]: 2024-07-21T01:52:36.100662133+02:00 INFO monito>
juli 21 01:52:36 alpha-legion systemd[1]: Started monitoring-agent.service - Monitoring agent.
```



