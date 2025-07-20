# Load env file and populate the indexer config template with secrets.
rindexer.yaml: rindexer.yaml.template .env
	export $$(grep -v '^#' .env | xargs) && envsubst < rindexer.yaml.template > rindexer.yaml

up: rindexer.yaml
	docker-compose up -d

build-up: rindexer.yaml
	docker-compose up -d --build

down:
	docker-compose down

clean:
	docker-compose down
	rm rindexer.yaml
