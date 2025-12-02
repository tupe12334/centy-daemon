mod crud;

pub use crud::{
    create_doc, delete_doc, get_doc, list_docs, update_doc, CreateDocOptions, CreateDocResult,
    DeleteDocResult, Doc, DocError, DocMetadata, UpdateDocOptions, UpdateDocResult,
};
