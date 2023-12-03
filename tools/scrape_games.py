#!/usr/bin/python3

import argparse
import datetime
import re
import sqlite3
import sys
import time
import urllib.error
import urllib.request

parser = argparse.ArgumentParser()
parser.add_argument('-d', '--database', default='data/games.sqlite3')
parser.add_argument('-c', '--competition', type=int, required=True)
args = parser.parse_args()

URL = 'https://www.codecup.nl/'
USER_AGENT = 'Tomek Czajka (tczajka@gmail.com)'

def download_page(page):
  time.sleep(2)

  result = None
  for attempt in range(3):
    print(f'Downloading {page}')
    try:
      result = urllib.request.urlopen(
          urllib.request.Request(
              URL + page,
              headers={'User-Agent': USER_AGENT}),
          timeout=10)
      break
    except urllib.error.URLError as error:
      print(f'ERROR: {error}')

  if result is None:
    print(f'Too many errors')
    sys.exit(1)

  assert result.getcode() == 200
  return result.read().decode('utf-8')

def download_competition(db, competition_id):
  page = download_page(f'competition.php?comp={competition_id}')

  # Get date.
  match = re.search(r'<th align=left>Date</th>\s*<td class=kimborder>([^<]*)</td>', page)
  assert match is not None
  date = datetime.datetime.strptime(match.group(1), '%a %b %d %Y').date().isoformat()

  db.execute('REPLACE INTO competition(id, date) VALUES (?, ?)', (competition_id, date))
  db.commit()
  
  # Get rounds.
  for match in re.finditer(r'<a href="competitionround.php\?cr=(\d+)">', page):
    download_round(db, competition_id, int(match.group(1)))

def download_round(db, competition_id, round_id):
  page = download_page(f'competitionround.php?cr={round_id}')
  
  db.execute('REPLACE INTO round(id, competition_id) VALUES (?, ?)', (round_id, competition_id))
  db.commit()

  for match in re.finditer(r"<a href='showgame.php\?ga=(\d+)'>", page):
    download_game(db, round_id, int(match.group(1)))

def download_game(db, round_id, game_id):
  # Check if we already have the game
  res = db.execute(
    '''
    SELECT COUNT(*) FROM game
    WHERE game.id = ?
    ''',
    (game_id,))

  if res.fetchone()[0] == 1:
    print(f'Already have game {game_id}')
    return

  page = download_page(f'showgame.php?ga={game_id}')

  # First
  match = re.search(r'<th align=left>First:&nbsp;&nbsp;</th><td class=kimborder>([^<]+)</td>', page)
  assert match is not None
  player_first = create_player(db, match.group(1))

  # Second
  match = re.search(r'<th align=left>Second:&nbsp;&nbsp;</th><td class=kimborder>([^<]+)</td>', page)
  assert match is not None
  player_second = create_player(db, match.group(1))

  # Moves
  match = re.search(r'new Sudoku\(\[[^]]*\], \[([^]]*)\]', page)
  if match is None:
    moves = []
    print('No moves')
  else:
    moves = match.group(1).split(',')
    # "Bb1" or "Bb1!"
    moves = [move[1:-1] for move in moves if len(move) <= 6]

  db.execute(
    '''
    REPLACE INTO game(id, round_id, player_first, player_second, moves)
    VALUES (?, ?, ?, ?, ?)
    ''',
    (game_id, round_id, player_first, player_second, ' '.join(moves)))
  db.commit()

def create_player(db, text):
  # text: "Tomek Czajka (#22928 Demon)"
  match = re.fullmatch(r'([^(]*) \(#(\d+) ([^)]*)\)', text)
  if match is not None:
    name = match.group(1)
    player_id = int(match.group(2))
    bot_name = match.group(3)
  elif text == 'Test Player A':
    name = text
    player_id = 0
    bot_name = 'test'
  else:
    assert False

  db.execute(
    '''
    REPLACE INTO player(id, name, bot_name)
    VALUES (?, ?, ?)
    ''',
    (player_id, name, bot_name))
  db.commit()
  return player_id


def main():
  db = sqlite3.connect(args.database)
  competition = args.competition
  download_competition(db, competition)
  db.close()
  print(f'Competition {competition} downloaded')

main()
