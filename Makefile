up:
	docker-compose up -d

down:
	docker-compose down

up-prod:
	docker compose -f docker-compose.prod.yaml --env-file .env up -d

down-prod:
	docker compose -f docker-compose.prod.yaml down
