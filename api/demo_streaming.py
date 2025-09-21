#!/usr/bin/env python3

import requests
import json
import time

def test_streaming():
    print("MusicBrainz Streaming Demo\n")
    print("This endpoint streams recommendations in real-time:")
    print("- Initial status updates")
    print("- Candidate tracks as they're scored")
    print("- Final sorted results\n")
    
    url = "http://localhost:3000/mb/recommend/stream"
    data = {
        "tracks": ["Bohemian Rhapsody Queen", "Stairway to Heaven Led Zeppelin"],
        "preferences": {
            "energy": 0.7,
            "obscurity": 0.3,
            "mood": 0.8
        }
    }
    
    print("Request:")
    print(json.dumps(data, indent=2))
    print("\nStreaming response:\n")
    
    try:
        # Make streaming request
        response = requests.post(url, json=data, stream=True, headers={'Accept': 'text/event-stream'})
        
        for line in response.iter_lines():
            if line:
                decoded_line = line.decode('utf-8')
                if decoded_line.startswith('data: '):
                    event_data = decoded_line[6:]  # Remove 'data: ' prefix
                    try:
                        event = json.loads(event_data)
                        if event['type'] == 'Status':
                            print(f"📊 {event['message']}")
                        elif event['type'] == 'Candidate':
                            track = event['track']
                            score = event['score']
                            print(f"🎵 Found: {track['name']} by {track['artist']} (score: {score:.3f})")
                        elif event['type'] == 'Complete':
                            print(f"\n✅ Complete! Top {len(event['tracks'])} recommendations:")
                            for i, track in enumerate(event['tracks'][:5]):
                                print(f"   {i+1}. {track['name']} by {track['artist']}")
                        elif event['type'] == 'Error':
                            print(f"❌ Error: {event['message']}")
                    except json.JSONDecodeError:
                        pass
    except requests.exceptions.ConnectionError:
        print("❌ Error: Could not connect to server. Make sure the API is running on port 3000.")
    except KeyboardInterrupt:
        print("\n\nStopped by user")

if __name__ == "__main__":
    test_streaming()