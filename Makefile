PYTHON_VERSION := 3.9.16

install-mac:
	brew install pyenv
	IS_INSTALLED=$(pyenv install --list | grep $(PYTHON_VERSION))
	if [ ! IS_INSTALLED ]; then \
		  pyenv install $(PYTHON_VERSION); \
	fi
	IS_VIRTUALENV=$(pyenv virtualenvs | grep aima-rs)
	if [ ! IS_VIRTUALENV ]; then \
		  pyenv virtualenv $(PYTHON_VERSION) aima-rs; \
	fi
	pyenv local aima-rs && \
		pip3 install --upgrade pip
	pyenv local aima-rs && \
		pip3 install torch torchvision torchaudio

run-nn-test-mac:
	pyenv local aima-rs && cd src && cargo run --bin nn-test

run-mcts-connect-four:
	pyenv local aima-rs && cd src && cargo run --profile production --bin mcts-connect-four

build:
	cd src && ./build-mac.sh

test:
	pyenv local aima-rs && cd src && cargo test --all

clean:
	pyenv local aima-rs && cd src && cargo clean

clean-virtualenv:
	pyenv uninstall aima-rs