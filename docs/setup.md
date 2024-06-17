# Setup

Rensen uses a simple setup which gets even more easy with the rensen-ctl.

## Keys

Rensen requires a form of private/public key authetication with the server.    
To generate the keys, do the following:   
   
### Generate keys: 
```bash
ssh-keygen -t ed25519
```
Make sure to generate without a passphrase, so that rensen can access it.

### Copy key over to host machine:
```bash
ssh-copy-id -i ed25519.pub user@x.x.x.x
```
This will copy the public key over to a machine that is going to be backupped.

## Rensen-ctl

The rensen-ctl can be used to do small tasks for the rensen.service.   
This involves adding new hosts, deleting and manual backups.

Now start the rensen-ctl.

```bash
sudo rensen-ctl
```

### Adding Host:

Add a new host by running the following `rensen command`:

```bash
add myserver
```
This will promt you for following interface where you will    
have to input information for the machine

```bash
addr: 192.168.22.88                                  # The ip address
user: root                                           # The user which rensen will use to backup
port (press enter for 22): 22                        # The ssh port (usually 22)
ssh-key path: /home/rensen-user/.ssh/myserver        # The private key 
source: /etc/mysql/important_stuff                   # The directory which is going to be backupped
destination:                                         # DEPRECATED (SKIP)
backupping schedule (Cron expression): * * * * * *   # Cron schedule (the schedule which rensend.service is follow for automatic backups)
```

## Run Manual Backups

You can either leave it up for rensend.service to do automatic (incremental) backups,     
or you can run them manually as follows:

### Run Incremental:
```bash
run inc myserver
```

### Run Full:
```bash
run full myserver
```




