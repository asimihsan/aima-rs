install-mac:
	brew install pyenv
	IS_INSTALLED=$(pyenv install --list | grep 3.9.16)
	if [ ! IS_INSTALLED ]; then \
		  pyenv install 3.9.16; \
	fi
	IS_VIRTUALENV=$(pyenv virtualenvs | grep aima-rs)
	if [ ! IS_VIRTUALENV ]; then \
		  pyenv virtualenv 3.9.16 aima-rs; \
	fi
	pyenv local aima-rs && \
		pip3 install --upgrade pip
	pyenv local aima-rs && \
		pip3 install torch torchvision torchaudio

run-nn-test-mac:
	pyenv local aima-rs && cd src && cargo run --bin nn-test