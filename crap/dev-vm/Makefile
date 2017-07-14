base:
	rsync -L -av ~/.zshrc ~/.vim* ~/rc ~/.gitconfig home/
	docker build --tag=sid-dev:latest .

run: base
	docker run -it sid-dev:latest /bin/zsh