FROM ghcr.io/joshstevens19/rindexer

COPY . /app/project_path

WORKDIR /app/project_path

CMD ["start", "indexer"]
