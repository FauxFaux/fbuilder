base:
	rsync -L -av ~/.zshrc ~/.vim* ~/rc ~/.gitconfig home/
	docker build --network=mope --tag=sid-dev:latest .

run: base
	docker run -v /var/tmp:/mnt --network=mope -it sid-dev:latest /bin/zsh
