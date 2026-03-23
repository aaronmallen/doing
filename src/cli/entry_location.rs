/// Tracks an entry's ID and section for locating it in the document.
#[derive(Clone, Debug)]
pub struct EntryLocation {
  pub id: String,
  pub section: String,
}
