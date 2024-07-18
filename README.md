# Monitoring agent

## Goals

The goals for the code is to develop a simple monitoring agent with the following functionality.
- Open tcp connection.
- Http request
- ++++

## Development

The codebase is most for learning more advanced rust code while trying to create something I can use in my home network.

## Testing

Testing currently only includes 
- tests for reading configuration files.
- test for full integration test running locally.

## Build

Install rust
Run `cargo build`

## Use

### Run

Run as non daemon `./monitoring_agent --config ./config.json`
Run as daemon `./monitoring_agent --config ./config.json --daemon`

### Arguments
| Argument  | Description | Default | 
| ------------- | ------------- | ------------- |
| config | Configuration file | /etc/monitoring_agent/config.json | 
| daemon | Run application as a daemon  | false | 
| test | Test a configuration file | false | 
| stdout | stdout file. Only used in daemon mode. | /var/log/monitoring_agent.out | 
| stderr | Test a configuration file | /var/log/monitoring_agent.err | 
| pidfile | Location of the pid file. Only in daemon mode. | /tmp/monitoring_agent.pid |

### Configuration file

#### Tcp monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| monitor.type | Type of monitor. Must be tcp | 
| monitor.host | Host/ip to connect to. | 
| monitor.port | Port to connect to. | 

#### Http monitoring

| Config  | Description | 
| ------------- | ------------- |
| name | Name for the monitoring | 
| schedule | Cron describing how often it should run | 
| monitor.type | Type of monitor. Must be http | 
| monitor.url | Url to make the request to. | 
| monitor.method | Method like post, put, delete, get, option, head | 
| monitor.body | Body to send | 
| monitor.headers | Headers to send | 

#### Example file

```
[
  {
        "name":"",
        "schedule": "*/10 * * * * *",
        "monitor": {
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
        "monitor": {
            "type": "tcp",
            "host": "127.0.0.1",
            "port": 139
        }
    }
]
```
