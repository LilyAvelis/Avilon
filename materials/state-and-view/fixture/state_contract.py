"""Small SQLite example of the contracts; not Katran's production schema."""
import hashlib
import json
import sqlite3

SCHEMA = """
PRAGMA foreign_keys=ON;
CREATE TABLE slots(scope TEXT, number INTEGER, label TEXT NOT NULL, PRIMARY KEY(scope, number));
CREATE TABLE assignments(scope TEXT, item_id INTEGER, slot INTEGER, PRIMARY KEY(scope,item_id),
 FOREIGN KEY(scope,slot) REFERENCES slots(scope,number));
CREATE TABLE revisions(scope TEXT PRIMARY KEY, counter INTEGER NOT NULL);
"""

def revision(db, scope):
    counter = db.execute("SELECT counter FROM revisions WHERE scope=?", (scope,)).fetchone()[0]
    definitions = db.execute("SELECT number,label FROM slots WHERE scope=? ORDER BY number", (scope,)).fetchall()
    return hashlib.sha256(json.dumps([scope, counter, definitions], separators=(",", ":")).encode()).hexdigest()

def reorder(db, scope, order, expected):
    db.execute("BEGIN IMMEDIATE")
    try:
        old = db.execute("SELECT number,label FROM slots WHERE scope=? ORDER BY number", (scope,)).fetchall()
        numbers = [n for n, _ in old]
        if sorted(order) != numbers or len(set(order)) != len(numbers):
            raise ValueError("invalid permutation")
        if revision(db, scope) != expected:
            raise ValueError("stale revision")
        labels = dict(old)
        for position, number in zip(numbers, order):
            db.execute("UPDATE slots SET label=? WHERE scope=? AND number=?", (labels[number], scope, position))
        # Values are bound. All target positions already exist for the FK.
        clauses = " ".join("WHEN ? THEN ?" for _ in order)
        pairs = [entry for position, old_number in zip(numbers, order) for entry in (old_number,position)]
        db.execute("UPDATE assignments SET slot=CASE slot " + clauses + " ELSE slot END WHERE scope=?", [*pairs,scope])
        db.execute("UPDATE revisions SET counter=counter+1 WHERE scope=?", (scope,))
        db.commit()
    except Exception:
        db.rollback()
        raise

def generation(db, scope):
    rows = db.execute("SELECT item_id,slot FROM assignments WHERE scope=? ORDER BY item_id", (scope,)).fetchall()
    return hashlib.sha256(json.dumps([revision(db,scope),rows],separators=(",", ":")).encode()).hexdigest()

def page(db, scope, limit, cursor=None):
    # Fixture assumes all items live in the same simple table; consumers bind
    # their own query/filter/sort and read generation/data from one snapshot.
    token = generation(db,scope)
    if cursor is not None and cursor[0] != token:
        raise ValueError("stale cursor")
    if cursor is None:
        after_slot,after_id = 0, 2**63-1
    else:
        _,after_slot,after_id = cursor
    rows = db.execute("""SELECT i.id, COALESCE(a.slot, 999) AS bucket FROM items i
      LEFT JOIN assignments a ON a.scope=? AND a.item_id=i.id
      WHERE i.scope=? AND (COALESCE(a.slot,999)>? OR (COALESCE(a.slot,999)=? AND i.id<?))
      ORDER BY bucket ASC, i.id DESC LIMIT ?""", (scope,scope,after_slot,after_slot,after_id,limit+1)).fetchall()
    more = len(rows)>limit
    rows = rows[:limit]
    return rows, ((token,rows[-1][1],rows[-1][0]) if more else None)
