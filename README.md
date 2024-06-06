# Redundancy Sentinel

Simple remote sync service.

## Build/Install

Install required dependencies:     
```bash
sudo ./dep.sh
```

Run the build script:     
```bash
sudo ./build.sh
```



## Usage/Examples

Conifguration for hosts can be changed inside the .yml file, or in the rensen ctl `sudo rensen`, where you have to use either the `add` or `mod` action.

/etc/rensen/hosts.yml 
```yaml
---
- hostname: dummy
  config: 
    user: dummy 
    identifier: 0.0.0.0
    port: 22
    key: /dummy/i/am/dum
    source: /dummy/i/am/dum
    destination: /dummy/i/am/dum
    cron_schedule: "* * 0 0 * * *"
```

You should specify the `cron_schedule` according to how often you want the `rensend.service` to run backup tasks.
