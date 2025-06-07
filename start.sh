#!/bin/bash

# Start the indexer in the background
rindexer start indexer &

# Start the API server
subindexer 