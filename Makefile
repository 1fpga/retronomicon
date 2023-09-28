.PHONY: db-reset db-rm db-start db-sync

db-reset: db-rm db-start db-sync

db-rm:
	@echo "Removing postgres container..."
	@docker stop $$(docker ps -q --filter ancestor=postgres) > /dev/null
	@docker rm $$(docker ps -a -q --filter ancestor=postgres) > /dev/null

db-start:
	@echo "Starting postgres container..."
	@docker run -e POSTGRES_USER=local_user \
	           -e POSTGRES_PASSWORD=mysecretpassword \
	           -e POSTGRES_DB=local_retronomicon \
	           -d -p 5432:5432 postgres
	@sleep 2

db-sync:
	@diesel migration revert -a
	@diesel migration run
	@planter $(DATABASE_URL)\?sslmode=disable -x __diesel_schema_migrations -o docs/database.puml
