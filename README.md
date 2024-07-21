# Monitoring agent

## Goals

The goals for the code is to develop a simple monitoring agent with the following functionality.
- Open tcp connection.
- Http request
- Commands

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

Run as non daemon `./monitoring_agent --config ./config.json --loggingfile ./logging.yml`
Run as daemon `./monitoring_agent --daemon`

### Arguments
| Argument  | Description | Default | 
| ------------- | ------------- | ------------- |
| config | Configuration file | /etc/monitoring_agent/config.json | 
| loggingfile | Logging file | /etc/monitoring_agent/logging.yml |
| daemon | Run application as a daemon  | false | 
| test | Test a configuration file | false | 
| stdout | stdout file. Only used in daemon mode. | /var/log/monitoring_agent.out | 
| stderr | Test a configuration file | /var/log/monitoring_agent.err | 
| pidfile | Location of the pid file. Only in daemon mode. | /tmp/monitoring_agent.pid |

### Configuration file

| Config  | Description | 
| ------------- | ------------- |
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

#### Command monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| details.type | Type of monitor. Must be command | 
| details.command | Command to run | 
| details.args | List of command arguments | 
| details.expected | Expected response | 

#### Example file

```
{
  "server": {
    "ip": "127.0.0.1",
    "port": 65000
  },
  "monitors": [
    {
          "name":"",
          "schedule": "*/10 * * * * *",
          "details": {
              "type": "http",
              "url": "http://post.com",
              "method": "post",
              "body": "body",
              "headers": {}
          }
      },
      {
          "name":"Netbios TCP",
          "schedule": "*/10 * * * * *",
          "details": {
              "type": "tcp",
              "host": "127.0.0.1",
              "port": 139
          }
      },
      {
          "name":"Systemctl memcached",
          "schedule": "*/5 * * * * *",
          "details": {
              "type": "command",
              "command": "systemctl",
              "args": ["show", "memcached.service", "--property=ActiveState"],
              "expected": "ActiveState=active\n"
          }
      }
  ]
}
```

## Setup as daemon
### Important
Type=forking This is important as the application forks a a daemon process. Without forking this setting this won't be identified and it will fail.

### Setup
Add the monitoring_agent command to /usr/local/bin
Add configuration file to /etc/monitoring_agent/config.json
Add logging file to /etc/monitoring_agent/logging.yml

Create a file in /etc/systemd/system/ called monitoring_agent.service
```
[Unit]
Description=Monitoring agent
DefaultDependencies=no
Before=shutdown.target

[Service]
ExecStart=/usr/local/bin/monitoring_agent --daemon
Type=forking
Restart=on-failure
TimeoutStartSec=10

[Install]
WantedBy=default.target
```

Run the command : systemctl daemon-reload
Run the command : systemctl enable monitoring_agent.service
Run the command : systemctl start monitoring_agent.service
Run the command : systemctl status monitoring_agent.service 
You should see the following result.
```
● monitoring_agent.service - Monitoring agent
     Loaded: loaded (/etc/systemd/system/monitoring_agent.service; enabled; preset: enabled)
     Active: active (running) since Sun 2024-07-21 01:52:36 CEST; 18h ago
    Process: 234467 ExecStart=/usr/local/bin/monitoring_agent --daemon (code=exited, status=0/SUCCESS)
   Main PID: 234470 (monitoring_agen)
      Tasks: 17 (limit: 18118)
     Memory: 15.6M (peak: 17.5M)
        CPU: 7min 19.565s
     CGroup: /system.slice/monitoring_agent.service
             └─234470 /usr/local/bin/monitoring_agent --daemon

juli 21 01:52:36 alpha-legion systemd[1]: Starting monitoring_agent.service - Monitoring agent...
juli 21 01:52:36 alpha-legion monitoring_agent[234467]: 2024-07-21T01:52:36.100662133+02:00 INFO monito>
juli 21 01:52:36 alpha-legion systemd[1]: Started monitoring_agent.service - Monitoring agent.
```



