#!/usr/bin/env python3
"""Zet de uitslag van de Kiesraad (EML_NL) om naar het invoerbestand van uitslagavond-tv.

Gebruik:
    python3 -I scripts/kiesraad-naar-invoer.py Resultaat_TK2025.eml.xml Totaaltelling_TK2025.eml.xml > cijfers/uitslag-tk2025.json

Beide bestanden staan in "Verkiezingsuitslag Tweede Kamer 2025 (Deel 1)" op data.overheid.nl
(https://data.overheid.nl/dataset/verkiezingsuitslag-tweede-kamer-2025, open data, CC0).
Zetels: het aantal gekozen kandidaten per lijst in het Resultaat (de vastgestelde uitslag).
Stemmen: de geldige stemmen per lijst in de Totaaltelling.
Kleuren en logo's staan niet in de open data: die levert de omroep zelf aan.
"""

import json
import sys
import xml.etree.ElementTree as ET

NS = {"e": "urn:oasis:names:tc:evs:schema:eml", "kr": "http://www.kiesraad.nl/extensions"}

# Sleutel en korte naam per lijstnummer in 2025. De volledige naam komt uit de EML.
KORT = {
    1: ("pvv", "PVV"),
    2: ("gl-pvda", "GL-PvdA"),
    3: ("vvd", "VVD"),
    4: ("nsc", "NSC"),
    5: ("d66", "D66"),
    6: ("bbb", "BBB"),
    7: ("cda", "CDA"),
    8: ("sp", "SP"),
    9: ("denk", "DENK"),
    10: ("pvdd", "PvdD"),
    11: ("fvd", "FvD"),
    12: ("sgp", "SGP"),
    13: ("cu", "CU"),
    14: ("volt", "Volt"),
    15: ("ja21", "JA21"),
    16: ("vrede-voor-dieren", "VvD"),
    17: ("bvnl", "BVNL"),
    18: ("bij1", "BIJ1"),
    19: ("lp", "LP"),
    20: ("50plus", "50PLUS"),
    21: ("piratenpartij", "Piraten"),
    22: ("fnp", "FNP"),
    23: ("vrij-verbond", "Vrij Verbond"),
    24: ("de-linie", "DE LINIE"),
    25: ("nl-plan", "NL PLAN"),
    26: ("ellect", "ELLECT"),
    27: ("pvdr", "PvdR"),
}


def zetels(resultaat):
    """Gekozen kandidaten per lijstnummer, uit het Resultaat (EML 520)."""
    wortel = ET.parse(resultaat).getroot()
    contest = wortel.find("e:Result/e:Election/e:Contest", NS)
    telling, lijst = {}, None
    for s in contest.findall("e:Selection", NS):
        a = s.find("e:AffiliationIdentifier", NS)
        if a is not None:
            lijst = int(a.get("Id"))
            telling.setdefault(lijst, 0)
        elif s.find("e:Candidate", NS) is not None and s.findtext("e:Elected", namespaces=NS) == "yes":
            telling[lijst] += 1
    return telling


def stemmen(totaaltelling):
    """Naam en geldige stemmen per lijstnummer, uit de Totaaltelling (EML 510)."""
    wortel = ET.parse(totaaltelling).getroot()
    verkiezing = wortel.find("e:Count/e:Election", NS)
    datum = verkiezing.findtext("e:ElectionIdentifier/kr:ElectionDate", namespaces=NS)
    naam = verkiezing.findtext("e:ElectionIdentifier/e:ElectionName", namespaces=NS)
    totaal = verkiezing.find("e:Contests/e:Contest/e:TotalVotes", NS)
    lijsten = {}
    for s in totaal.findall("e:Selection", NS):
        a = s.find("e:AffiliationIdentifier", NS)
        if a is not None:
            lijsten[int(a.get("Id"))] = (a.findtext("e:RegisteredName", namespaces=NS), int(s.findtext("e:ValidVotes", namespaces=NS)))
    return naam, datum, lijsten


def main(resultaat, totaaltelling):
    gekozen = zetels(resultaat)
    naam, datum, lijsten = stemmen(totaaltelling)
    assert sum(gekozen.values()) == 150, sum(gekozen.values())
    assert set(lijsten) == set(KORT), sorted(lijsten)
    partijen = [
        {"sleutel": KORT[n][0], "naam": lijsten[n][0], "kort": KORT[n][1]}
        for n in sorted(lijsten)
    ]
    invoer = {
        "verkiezing": {"naam": naam, "datum": datum},
        "partijen": partijen,
        "sets": [
            {
                "id": "tk2025",
                "soort": "uitslag",
                "label": "Uitslag 2025",
                "tijd": datum,
                "bron": "Kiesraad, open data (CC0)",
                "zetels": {KORT[n][0]: z for n, z in sorted(gekozen.items()) if z > 0},
                "stemmen": {KORT[n][0]: lijsten[n][1] for n in sorted(lijsten)},
            }
        ],
        "vergelijk_met": "tk2025",
    }
    json.dump(invoer, sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main(*sys.argv[1:3])
