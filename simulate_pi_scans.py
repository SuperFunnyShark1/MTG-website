import random
import sys
import time
import requests

DEFAULT_SERVER_URL = "http://localhost:3001"
DELAY_SECONDS = 1.5  # Time between simulated card scans

SAMPLE_CARDS = [
    {
        "set_code": "mh3",
        "collector_number": "0129",
        "lang": "en",
        "is_foil": False,
        "name": "Phlage, Titan of Fire's Fury",
        "name_en": "Phlage, Titan of Fire's Fury",
        "name_es": None,
        "rarity": "m",
        "colors": ["R", "W"],
        "bin_number": 1,
        "source": "scryfall",
    },
    {
        "set_code": "blb",
        "collector_number": "0103",
        "lang": "es",
        "is_foil": True,
        "name": "Maha, su plumaje nocturno",
        "name_en": "Maha, Its Feathers Night",
        "name_es": "Maha, su plumaje nocturno",
        "rarity": "m",
        "colors": ["B"],
        "bin_number": 1,
        "source": "scryfall",
    },
    {
        "set_code": "ltr",
        "collector_number": "0246",
        "lang": "en",
        "is_foil": True,
        "name": "The One Ring",
        "name_en": "The One Ring",
        "name_es": "El Anillo Único",
        "rarity": "m",
        "colors": [],
        "bin_number": 1,
        "source": "scryfall",
    },
    {
        "set_code": "fdn",
        "collector_number": "0027",
        "lang": "en",
        "is_foil": False,
        "name": "Day of Judgment",
        "name_en": "Day of Judgment",
        "name_es": "Día del juicio",
        "rarity": "r",
        "colors": ["W"],
        "bin_number": 2,
        "source": "scryfall",
    },
    {
        "set_code": "dsk",
        "collector_number": "0056",
        "lang": "es",
        "is_foil": False,
        "name": "Entidad de la tormenta",
        "name_en": "Entity Tracker",
        "name_es": "Entidad de la tormenta",
        "rarity": "r",
        "colors": ["U"],
        "bin_number": 2,
        "source": "scryfall",
    },
    {
        "set_code": "blb",
        "collector_number": "0173",
        "lang": "es",
        "is_foil": False,
        "name": "Recluta patazarpa",
        "name_en": "Pawpatch Recruit",
        "name_es": "Recluta patazarpa",
        "rarity": "r",
        "colors": ["G"],
        "bin_number": 3,
        "source": "scryfall",
    },
    {
        "set_code": "m11",
        "collector_number": "0149",
        "lang": "es",
        "is_foil": True,
        "name": "Relámpago",
        "name_en": "Lightning Bolt",
        "name_es": "Relámpago",
        "rarity": "u",
        "colors": ["R"],
        "bin_number": 4,
        "source": "local_db",
    },
    {
        "set_code": "cmm",
        "collector_number": "0388",
        "lang": "en",
        "is_foil": False,
        "name": "Sol Ring",
        "name_en": "Sol Ring",
        "name_es": "Anillo solar",
        "rarity": "u",
        "colors": [],
        "bin_number": 4,
        "source": "scryfall",
    },
    {
        "set_code": "dmr",
        "collector_number": "0041",
        "lang": "es",
        "is_foil": False,
        "name": "Contrahechizo",
        "name_en": "Counterspell",
        "name_es": "Contrahechizo",
        "rarity": "c",
        "colors": ["U"],
        "bin_number": 5,
        "source": "local_db",
    },
    {
        "set_code": "neo",
        "collector_number": "0036",
        "lang": "en",
        "is_foil": False,
        "name": "The Wandering Emperor",
        "name_en": "The Wandering Emperor",
        "name_es": "La Emperatriz Errante",
        "rarity": "m",
        "colors": ["W"],
        "bin_number": 6,
        "source": "scryfall",
    },
]


def check_server(base_url: str) -> bool:
    try:
        resp = requests.get(f"{base_url}/api/health", timeout=3.0)
        if resp.status_code == 200:
            print(f"[OK] Connected to {base_url} -> {resp.json()}")
            return True
        print(f"[ERROR] Health check returned HTTP {resp.status_code}")
        return False
    except requests.RequestException as exc:
        print(f"[ERROR] Cannot reach {base_url}: {exc}")
        return False


def send_card(base_url: str, card: dict) -> None:
    payload = {**card, "image_url": None}  # Let Rust fetch artwork & CLP price from Scryfall
    try:
        resp = requests.post(f"{base_url}/api/scans", json=payload, timeout=3.0)
        if resp.status_code == 200:
            data = resp.json()
            foil = " [FOIL]" if card["is_foil"] else ""
            print(
                f"Sent: [{card['set_code'].upper()} #{card['collector_number']}]{foil} "
                f"{card['name']} -> Bin #{card['bin_number']} | Stock: {data.get('quantity')}"
            )
        else:
            print(f"[FAIL] HTTP {resp.status_code}: {resp.text}")
    except requests.RequestException as exc:
        print(f"[FAIL] Network error: {exc}")


def main() -> None:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    flags = {a for a in sys.argv[1:] if a.startswith("--")}

    base_url = (args[0] if args else DEFAULT_SERVER_URL).rstrip("/")
    continuous = "--loop" in flags

    if not check_server(base_url):
        sys.exit(1)

    print(f"Simulating Raspberry Pi scans (delay={DELAY_SECONDS}s)... Press Ctrl+C to stop.\n")

    try:
        if continuous:
            while True:
                card = random.choice(SAMPLE_CARDS)
                send_card(base_url, card)
                time.sleep(DELAY_SECONDS)
        else:
            # Send all sample cards once, plus 3 random duplicates to test stock stacking
            batch = SAMPLE_CARDS + random.sample(SAMPLE_CARDS, 3)
            for card in batch:
                send_card(base_url, card)
                time.sleep(DELAY_SECONDS)
            print("\nDone! Run with --loop to keep sending random cards continuously.")
    except KeyboardInterrupt:
        print("\nSimulation stopped.")


if __name__ == "__main__":
    main()
    
    
    # !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    # code to run python simulate_pi_scans.py http://192.168.43.120:3001