# Redundancy Sentinel

Simple Redundancy Sentinel Service.

## Build/Install

Clone Repo:
```bash
git clone https://github.com/bampam/rensen
```
or
```bash
git clone https://git.sditto.org/rensen.git
```

Install required dependencies (cargo, ssl, gcc):     
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

Please refer to [setup.md](https://github.com/bampam/rensen/blob/main/docs/setup.md) for full setup guide.


