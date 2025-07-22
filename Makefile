# Load env file and populate the indexer config template with secrets.
rindexer.yaml: rindexer.yaml.template .env
	export $$(grep -v '^#' .env | xargs) && envsubst < rindexer.yaml.template > rindexer.yaml

up: rindexer.yaml
	docker-compose up -d

down:
	docker-compose down

clean:
	docker-compose down
	rm rindexer.yaml

up-prod: rindexer.yaml
	docker compose -f docker-compose.prod.yaml --env-file .env up -d

down-prod:
	docker compose -f docker-compose.prod.yaml down
