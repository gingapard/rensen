sudo apt install libssl-dev

cargo build --manifest-path daemon/Cargo.toml --release
sudo cp daemon/target/release/rensend /usr/bin/rensend
sudo cp daemon/service/rensend.service /etc/systemd/system/rensend.service

cargo build --manifest-path ctl/Cargo.toml --release
sudo cp ctl/target/release/rensen-ctl /usr/bin/rensen

sudo mkdir -p /etc/rensen
sudo cp assets/rensen_config.yml /etc/rensen
sudo cp assets/hosts.yml /etc/rensen
