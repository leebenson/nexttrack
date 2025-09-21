#!/bin/bash

echo "Testing MusicBrainz streaming recommendations endpoint..."
echo ""

# Use curl with -N to disable buffering for SSE
curl -N -X POST http://localhost:3000/mb/recommend/stream \
  -H "Content-Type: application/json" \
  -d '{
    "tracks": ["Bohemian Rhapsody Queen", "Stairway to Heaven Led Zeppelin"],
    "preferences": {
      "energy": 0.7,
      "obscurity": 0.3,
      "mood": 0.8
    }
  }'