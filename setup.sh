cp daemon/target/release/rensend /usr/bin/rensend
cp daemon/service/rensend.service /etc/systemd/system/rensend.service
cp ctl/target/release/rensen-ctl /usr/bin/rensen
mkdir -p /etc/rensen
cp assets/rensen_config.yml /etc/rensen
cp assets/hosts.yml /etc/rensen
