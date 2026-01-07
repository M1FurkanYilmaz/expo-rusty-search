use tantivy::schema::*;
use tantivy::{Index, doc, collector::TopDocs, TantivyDocument};
use tantivy::query::QueryParser;
use std::ffi::{CStr, CString};
use std::sync::{Mutex, OnceLock};
use std::path::Path;

// Global static index that persists between function calls
static GLOBAL_INDEX: OnceLock<Mutex<Option<Index>>> = OnceLock::new();

// Helper function to get or create the schema
fn get_schema() -> (Schema, Field, Field) {
    let mut schema_builder = Schema::builder();
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let body = schema_builder.add_text_field("body", TEXT | STORED);
    let schema = schema_builder.build();
    (schema, title, body)
}

// Helper function to get the global index mutex
fn get_index_mutex() -> &'static Mutex<Option<Index>> {
    GLOBAL_INDEX.get_or_init(|| Mutex::new(None))
}

/// Initialize or load the index from disk
/// Parameters:
///   - index_path_ptr: C string pointer to the directory path where index should be stored
/// Returns: Success or error message
#[unsafe(no_mangle)]
pub unsafe extern "C" fn initialize_index(
    index_path_ptr: *const std::os::raw::c_char,
) -> *const std::os::raw::c_char {
    // Convert C string to Rust string
    let path_cstr = unsafe { CStr::from_ptr(index_path_ptr) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8 in path").unwrap().into_raw(),
    };

    let index_path = Path::new(path_str);
    let (schema, _, _) = get_schema();

    // Try to open existing index, or create a new one
    let index = if index_path.exists() {
        // Load existing index from disk
        match Index::open_in_dir(index_path) {
            Ok(idx) => {
                let doc_count = match idx.reader() {
                    Ok(reader) => reader.searcher().num_docs(),
                    Err(_) => 0,
                };
                let msg = format!("Index loaded successfully from disk. Documents: {}", doc_count);
                
                // Store in global variable
                let index_mutex = get_index_mutex();
                if let Ok(mut guard) = index_mutex.lock() {
                    *guard = Some(idx);
                }
                
                return CString::new(msg).unwrap().into_raw();
            }
            Err(e) => {
                return CString::new(format!("Error loading index: {}", e)).unwrap().into_raw();
            }
        }
    } else {
        // Create new index on disk
        match Index::create_in_dir(index_path, schema) {
            Ok(idx) => {
                // Store in global variable
                let index_mutex = get_index_mutex();
                if let Ok(mut guard) = index_mutex.lock() {
                    *guard = Some(idx);
                }
                
                CString::new("New index created successfully on disk").unwrap().into_raw()
            }
            Err(e) => {
                return CString::new(format!("Error creating index: {}", e)).unwrap().into_raw();
            }
        }
    };

    index
}

/// Function to add a document to the index
/// Parameters:
///   - title_ptr: C string pointer to the document title
///   - body_ptr: C string pointer to the document body
/// Returns: Success or error message
#[unsafe(no_mangle)]
pub unsafe extern "C" fn add_document(
    title_ptr: *const std::os::raw::c_char,
    body_ptr: *const std::os::raw::c_char,
) -> *const std::os::raw::c_char {
    // Convert C strings to Rust strings
    let title_cstr = unsafe { CStr::from_ptr(title_ptr) };
    let title_str = match title_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8 in title").unwrap().into_raw(),
    };

    let body_cstr = unsafe { CStr::from_ptr(body_ptr) };
    let body_str = match body_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8 in body").unwrap().into_raw(),
    };

    // Get the global index
    let index_mutex = get_index_mutex();
    let mut index_guard = match index_mutex.lock() {
        Ok(guard) => guard,
        Err(_) => return CString::new("Error: Failed to lock index").unwrap().into_raw(),
    };

    let index = match index_guard.as_ref() {
        Some(idx) => idx,
        None => return CString::new("Error: Index not initialized. Call initialize_index() first").unwrap().into_raw(),
    };

    // Get schema fields
    let (_, title_field, body_field) = get_schema();

    // Create index writer
    let mut index_writer: tantivy::IndexWriter<TantivyDocument> = match index.writer(50_000_000) {
        Ok(writer) => writer,
        Err(e) => return CString::new(format!("Error creating writer: {}", e)).unwrap().into_raw(),
    };

    // Add the document
    if let Err(e) = index_writer.add_document(doc!(
        title_field => title_str,
        body_field => body_str
    )) {
        return CString::new(format!("Error adding document: {}", e)).unwrap().into_raw();
    }

    // Commit the document to make it searchable and persist to disk
    if let Err(e) = index_writer.commit() {
        return CString::new(format!("Error committing document: {}", e)).unwrap().into_raw();
    }

    CString::new("Document added successfully").unwrap().into_raw()
}

/// Function to search through all added documents
/// Parameters:
///   - query_ptr: C string pointer to the search query
/// Returns: Search results or error message
#[unsafe(no_mangle)]
pub unsafe extern "C" fn search_basic(query_ptr: *const std::os::raw::c_char) -> *const std::os::raw::c_char {
    // Convert C string to Rust string
    let c_str = unsafe { CStr::from_ptr(query_ptr) };
    let query_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8").unwrap().into_raw(),
    };

    // Get the global index
    let index_mutex = get_index_mutex();
    let index_guard = match index_mutex.lock() {
        Ok(guard) => guard,
        Err(_) => return CString::new("Error: Failed to lock index").unwrap().into_raw(),
    };

    let index = match index_guard.as_ref() {
        Some(idx) => idx,
        None => return CString::new("Error: Index not initialized. Call initialize_index() first").unwrap().into_raw(),
    };

    // Get schema fields
    let (_, title, body) = get_schema();

    // Create a reader and searcher
    let reader: tantivy::IndexReader = match index.reader() {
        Ok(r) => r,
        Err(e) => return CString::new(format!("Error creating reader: {}", e)).unwrap().into_raw(),
    };
    let searcher = reader.searcher();

    // Parse the query
    let query_parser = QueryParser::for_index(&index, vec![title, body]);
    let query = match query_parser.parse_query(query_str) {
        Ok(q) => q,
        Err(e) => return CString::new(format!("Error parsing query: {}", e)).unwrap().into_raw(),
    };

    // Execute the search
    let top_docs: Vec<(f32, tantivy::DocAddress)> = match searcher.search(&query, &TopDocs::with_limit(10)) {
        Ok(docs) => docs,
        Err(e) => return CString::new(format!("Error searching: {}", e)).unwrap().into_raw(),
    };

    // Format the results
    let mut results = String::from("Search Results:\n\n");
    
    if top_docs.is_empty() {
        results.push_str("No results found.");
    } else {
        for (score, doc_address) in top_docs {
            if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address) {
                let title_text = retrieved_doc
                    .get_first(title)
                    .and_then(|v| v.as_str())
                    .unwrap_or("No title");
                let body_text = retrieved_doc
                    .get_first(body)
                    .and_then(|v| v.as_str())
                    .unwrap_or("No body");
                
                results.push_str(&format!(
                    "Score: {:.2}\nTitle: {}\nBody: {}\n\n",
                    score, title_text, body_text
                ));
            }
        }
    }

    // Return results as C-compatible string
    CString::new(results).unwrap().into_raw()
}

/// Get the number of documents in the index
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_document_count() -> *const std::os::raw::c_char {
    let index_mutex = get_index_mutex();
    let index_guard = match index_mutex.lock() {
        Ok(guard) => guard,
        Err(_) => return CString::new("Error: Failed to lock index").unwrap().into_raw(),
    };

    let index = match index_guard.as_ref() {
        Some(idx) => idx,
        None => return CString::new("0").unwrap().into_raw(),
    };

    let count = match index.reader() {
        Ok(reader) => reader.searcher().num_docs(),
        Err(_) => 0,
    };

    CString::new(format!("{}", count)).unwrap().into_raw()
}

// Helper function to free the returned string from C side
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_string(ptr: *mut std::os::raw::c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// cbindgen:ignore
#[cfg(target_os = "android")]
pub mod android {
    use crate::*; // Access your core functions
    use jni::JNIEnv;
    use jni::objects::{JClass, JString};
    use jni::sys::jstring;
    use std::ffi::{CStr, CString};

    // Helper to convert Rust *const c_char to Java jstring and free the pointer
    unsafe fn rust_ptr_to_jstring(env: &mut JNIEnv, ptr: *const std::os::raw::c_char) -> jstring {
        if ptr.is_null() {
            return env.new_string("").unwrap().into_raw();
        }
        let c_str = CStr::from_ptr(ptr);
        let output = env.new_string(c_str.to_str().unwrap_or("")).unwrap();
        
        // Very Important: Free the string memory in Rust after copying it to Java
        free_string(ptr as *mut std::os::raw::c_char);
        
        output.into_raw()
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Java_expo_modules_rustysearch_ExpoRustySearchModule_initializeIndex(
        mut env: JNIEnv,
        _class: JClass,
        index_path: JString,
    ) -> jstring {
        let path: String = env.get_string(&index_path).unwrap().into();
        let c_path = CString::new(path).unwrap();
        
        let result_ptr = initialize_index(c_path.as_ptr());
        rust_ptr_to_jstring(&mut env, result_ptr)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Java_expo_modules_rustysearch_ExpoRustySearchModule_addDocument(
        mut env: JNIEnv,
        _class: JClass,
        title: JString,
        body: JString,
    ) -> jstring {
        let r_title: String = env.get_string(&title).unwrap().into();
        let r_body: String = env.get_string(&body).unwrap().into();
        
        let c_title = CString::new(r_title).unwrap();
        let c_body = CString::new(r_body).unwrap();
        
        let result_ptr = add_document(c_title.as_ptr(), c_body.as_ptr());
        rust_ptr_to_jstring(&mut env, result_ptr)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Java_expo_modules_rustysearch_ExpoRustySearchModule_searchBasic(
        mut env: JNIEnv,
        _class: JClass,
        query: JString,
    ) -> jstring {
        let r_query: String = env.get_string(&query).unwrap().into();
        let c_query = CString::new(r_query).unwrap();
        
        let result_ptr = search_basic(c_query.as_ptr());
        rust_ptr_to_jstring(&mut env, result_ptr)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Java_expo_modules_rustysearch_ExpoRustySearchModule_getDocumentCount(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let result_ptr = get_document_count();
        rust_ptr_to_jstring(&mut env, result_ptr)
    }
}