# Vagrantfile to provision a Debian VM that builds the project and runs contrib/install.sh
Vagrant.configure("2") do |config|
  config.vm.box = "generic/debian12"
  config.vm.hostname = "checkvpn-dev"

  config.vm.provider "virtualbox" do |vb|
    vb.memory = 2048
    vb.cpus = 2
  end

  # Sync the repository into /vagrant in the VM
  # VirtualBox shared folder is the default for vagrant boxes
  config.vm.synced_folder ".", "/vagrant", type: "virtualbox"

  # Run a shell provisioner that builds and installs the project
  config.vm.provision "shell", path: "scripts/vagrant_provision.sh"
end
