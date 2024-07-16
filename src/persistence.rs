use crate::storage::GeoDatabase;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, ErrorKind, Write};

const WAL_FILE: &str = "wal.log"; // The WAL logs each write operation (e.g., adding a geospatial point) to disk.
const SNAPSHOT_FILE: &str = "snapshot.bincode"; // A snapshot is a complete copy of the database at a certain point in time.

#[derive(Serialize, Deserialize)]
pub enum WalEntry {
    // Make this enum public
    GeoAdd {
        key: String,
        coords: Vec<(f64, f64)>,
    },
}

pub struct Persistence {
    wal_writer: BufWriter<File>,
}

impl Persistence {
    pub fn new() -> io::Result<Self> {
        let wal_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(WAL_FILE)?;
        let wal_writer = BufWriter::new(wal_file);

        Ok(Persistence { wal_writer })
    }

    pub fn log_entry(&mut self, entry: WalEntry) -> io::Result<()> {
        let entry_bytes =
            bincode::serialize(&entry).map_err(|e| io::Error::new(ErrorKind::Other, e))?; // convert to bytes
        self.wal_writer.write_all(&entry_bytes)?;
        self.wal_writer.write_all(b"\n")?;
        self.wal_writer.flush()
    }

    pub fn load_wal(db: &mut GeoDatabase) -> io::Result<()> {
        let wal_file = File::open(WAL_FILE)?;
        let reader = BufReader::new(wal_file);

        for line in reader.lines() {
            let line = line?;
            let entry: WalEntry = bincode::deserialize(line.as_bytes())
                .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
            match entry {
                WalEntry::GeoAdd { key, coords } => {
                    db.geo_add(key, coords);
                }
            }
        }
        Ok(())
    }

    pub fn create_snapshot(db: &GeoDatabase) -> io::Result<()> {
        let snapshot_file = File::create(SNAPSHOT_FILE)?;
        let writer = BufWriter::new(snapshot_file);
        bincode::serialize_into(writer, db).map_err(|e| io::Error::new(ErrorKind::Other, e))
    }

    pub fn load_snapshot() -> io::Result<GeoDatabase> {
        let snapshot_file = File::open(SNAPSHOT_FILE)?;
        let reader = BufReader::new(snapshot_file);
        let db: GeoDatabase =
            bincode::deserialize_from(reader).map_err(|e| io::Error::new(ErrorKind::Other, e))?;
        Ok(db)
    }
}
