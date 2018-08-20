use std::io::{self, BufRead, BufReader, Lines, Read};

#[derive(Debug)]
pub enum FastaError {
    Parse(String),
    Io(io::Error),
}

#[derive(Clone)]
pub struct Record {
    id: String,
    seq: String,
    entry_string: String,
}

impl Record {
    /// Creates a new Record from a &String containing a fasta entry.
    /// Returns None if the string is empty.
    pub fn new(entry_string: &String) -> Result<Record, FastaError> {
        let mut lines_iter = entry_string.split('\n');

        let id = lines_iter.next()
            .ok_or(FastaError::Parse("Parsing error!".to_owned()))
            .and_then(|l| get_id_from_defline(&l))?
            .to_string();

        let mut seq = String::new();

        for line in lines_iter {
            seq.push_str(line);
        }

        Ok(Record {
            id: id,
            seq: seq,
            entry_string: entry_string.to_owned(),
        })
    }

    pub fn id(&self) -> &str { &self.id }
    pub fn seq(&self) -> &str { &self.seq }
    pub fn to_string(&self) -> &str { &self.entry_string }
}

/// Takes a fasta defline (e.g., ">seqID sequence desccription") and returns the
/// ID of the entry (e.g., "SeqID")
///
/// # Errors
/// Returns Err("Parsing error!") if an ID cannot be found in the defline, e.g.,
/// if the defline is empty or there is a space after the ">"
fn get_id_from_defline(defline: &str) -> Result<&str, FastaError> {
    defline.split_whitespace().next() // get the first word
        .ok_or(FastaError::Parse("Can't parse defline".to_owned()))
        .map(|w| w.trim_left_matches('>')) // trim the '>' delimiter
}

pub struct Reader<T> {
    lines_iter: Lines<BufReader<T>>,
    current_entry: Record,
}

impl<T: Read> Reader<T> {
    pub fn new(file: T) -> Reader<T> {
        Reader {
            lines_iter: BufReader::new(file).lines(),
            current_entry: Record {
                id: String::new(),
                seq: String::new(),
                entry_string: String::new(),
            }
        }
    }
}

impl<T: Read> Iterator for Reader<T> {
    type Item = Result<Record, FastaError>;

    fn next(&mut self) -> Option<Result<Record, FastaError>> {
        while let Some(result) = self.lines_iter.next() {
            let line = match result {
                Ok(r) => r,
                Err(e) => return Some(Err(FastaError::Io(e))),
            };

            if line.starts_with(">") {
                if self.current_entry.entry_string != "" {
                    // we have reached the beginning of a new entry, so we move
                    // the instance of Record representing the current one to a
                    // new variable, start a new instance of Record for the new
                    // one, and then return the completed one.
                    let finished_entry = self.current_entry.clone();
                    self.current_entry = Record {
                        id: match get_id_from_defline(&line) {
                            Ok(id) => id.to_string(),
                            Err(e) => return Some(Err(e)),
                        },
                        seq: String::new(),
                        entry_string: String::from(line),
                    };
                    return Some(Ok(finished_entry));
                } else {
                    // we're on the first line, so don't return anything; just
                    // update the entry string and id.
                    self.current_entry.entry_string.push_str(&line);
                    self.current_entry.id = match get_id_from_defline(&line) {
                        Ok(id) => id.to_string(),
                        Err(e) => return Some(Err(e)),
                    }
                }
            } else { // line is not the defline
                self.current_entry.entry_string.push_str(&line);
                self.current_entry.seq.push_str(&line.trim());
            }
        }
        
        if self.current_entry.entry_string != "" {
            let finished_entry = self.current_entry.clone();
            self.current_entry.entry_string = String::new();
            Some(Ok(finished_entry))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn fasta_record() {
        let entry_string = ">id\nACTG\nAAAA\nACGT".to_string();
        let rec = Record::new(&entry_string).unwrap();
        assert_eq!(rec.id(), "id".to_string());
        assert_eq!(rec.seq(), "ACTGAAAAACGT".to_string());
        assert_eq!(rec.to_string(), entry_string);
    }
}
