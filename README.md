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

Setup rensen-daemon and file structure:
```bash
sudo ./setup.sh
```

## Tech

Rensen uses the SFTP protocol to transfer/backup files from machines.    
All files will be archived and compressed into the tar.gz format  
when copied, and they will be placed in a structual way with records   
about the content (mtime, size etc.)

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
    cron_schedule: "* * * * * *"
```

You should specify the `cron_schedule` according to how often you want the `rensend.service` to run backup tasks.   
Generating a ed25519 public/private key is recommended.

### Run Manuel Backups 

Enter host-adding interface
```bash
add myserver
```

Run full backup of machine
```bash
run myserver full
```

