
# Load .env file if it exists
ifneq (,$(wildcard .env))
    include .env
    export
endif

rindexer.yaml: rindexer.yaml.template .env
	export $$(grep -v '^#' .env | xargs) && envsubst < rindexer.yaml.template > rindexer.yaml


up: rindexer.yaml
	docker-compose up -d

down:
	docker-compose down

clean:
	docker-compose down
	rm rindexer.yaml
