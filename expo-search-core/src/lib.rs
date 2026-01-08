use tantivy::schema::*;
use tantivy::{Index, doc, collector::TopDocs, TantivyDocument};
use tantivy::query::QueryParser;
use std::ffi::{CStr, CString};
use std::sync::{Mutex, OnceLock};
use serde_json;

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

/// Initialize a new in-memory index
/// Returns: Success or error message
#[unsafe(no_mangle)]
pub unsafe extern "C" fn initialize_index() -> *const std::os::raw::c_char {
    let (schema, _, _) = get_schema();
    
    // Create new in-memory index
    let index = Index::create_in_ram(schema);
    
    // Store in global variable
    let index_mutex = get_index_mutex();
    if let Ok(mut guard) = index_mutex.lock() {
        *guard = Some(index);
        return CString::new("Index initialized successfully in memory").unwrap().into_raw();
    }
    
    CString::new("Error: Failed to initialize index").unwrap().into_raw()
}

/// Clear the index and reinitialize it
/// This is useful when you want to rebuild the index from scratch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn clear_index() -> *const std::os::raw::c_char {
    unsafe { initialize_index() }
}

/// Add a single document to the index
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
    let index_guard = match index_mutex.lock() {
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

    // Commit the document to make it searchable
    if let Err(e) = index_writer.commit() {
        return CString::new(format!("Error committing document: {}", e)).unwrap().into_raw();
    }

    CString::new("Document added successfully").unwrap().into_raw()
}

/// Add multiple documents at once from a JSON array
/// Parameters:
///   - json_ptr: C string pointer to JSON array like [{"title":"..","body":".."}, ...]
/// Returns: Success message with count or error message
#[unsafe(no_mangle)]
pub unsafe extern "C" fn add_documents_bulk(
    json_ptr: *const std::os::raw::c_char,
) -> *const std::os::raw::c_char {
    // Convert C string to Rust string
    let json_cstr = unsafe { CStr::from_ptr(json_ptr) };
    let json_str = match json_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("Error: Invalid UTF-8 in JSON").unwrap().into_raw(),
    };

    // Parse JSON
    let docs: Vec<serde_json::Value> = match serde_json::from_str(json_str) {
        Ok(d) => d,
        Err(e) => return CString::new(format!("Error parsing JSON: {}", e)).unwrap().into_raw(),
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
    let (_, title_field, body_field) = get_schema();

    // Create index writer
    let mut index_writer: tantivy::IndexWriter<TantivyDocument> = match index.writer(50_000_000) {
        Ok(writer) => writer,
        Err(e) => return CString::new(format!("Error creating writer: {}", e)).unwrap().into_raw(),
    };

    let mut count = 0;
    for doc_json in docs {
        let title = doc_json["title"].as_str().unwrap_or("");
        let body = doc_json["body"].as_str().unwrap_or("");

        if let Err(e) = index_writer.add_document(doc!(
            title_field => title,
            body_field => body
        )) {
            return CString::new(format!("Error adding document: {}", e)).unwrap().into_raw();
        }
        count += 1;
    }

    // Commit all documents
    if let Err(e) = index_writer.commit() {
        return CString::new(format!("Error committing documents: {}", e)).unwrap().into_raw();
    }

    CString::new(format!("{} documents added successfully", count)).unwrap().into_raw()
}

/// Search through all indexed documents
/// Parameters:
///   - query_ptr: C string pointer to the search query
/// Returns: JSON array of search results
#[unsafe(no_mangle)]
pub unsafe extern "C" fn search_documents(
    query_ptr: *const std::os::raw::c_char,
) -> *const std::os::raw::c_char {
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
        None => return CString::new("[]").unwrap().into_raw(), // Return empty array if not initialized
    };

    // Get schema fields
    let (_, title, body) = get_schema();

    // Create a reader and searcher
    let reader: tantivy::IndexReader = match index.reader() {
        Ok(r) => r,
        Err(e) => return CString::new(format!("{{\"error\": \"{}\"}}", e)).unwrap().into_raw(),
    };
    let searcher = reader.searcher();

    // Parse the query
    let query_parser = QueryParser::for_index(&index, vec![title, body]);
    let query = match query_parser.parse_query(query_str) {
        Ok(q) => q,
        Err(e) => return CString::new(format!("{{\"error\": \"{}\"}}", e)).unwrap().into_raw(),
    };

    // Execute the search
    let top_docs: Vec<(f32, tantivy::DocAddress)> = match searcher.search(&query, &TopDocs::with_limit(10)) {
        Ok(docs) => docs,
        Err(e) => return CString::new(format!("{{\"error\": \"{}\"}}", e)).unwrap().into_raw(),
    };

    // Build JSON results
    let mut results = Vec::new();
    
    for (score, doc_address) in top_docs {
        if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address) {
            let title_text = retrieved_doc
                .get_first(title)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let body_text = retrieved_doc
                .get_first(body)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            results.push(serde_json::json!({
                "title": title_text,
                "body": body_text,
                "score": score
            }));
        }
    }

    let json_result = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
    CString::new(json_result).unwrap().into_raw()
}

/// Get the number of documents in the index
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_document_count() -> *const std::os::raw::c_char {
    let index_mutex = get_index_mutex();
    let index_guard = match index_mutex.lock() {
        Ok(guard) => guard,
        Err(_) => return CString::new("0").unwrap().into_raw(),
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
    use crate::*;
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
    ) -> jstring {
        let result_ptr = initialize_index();
        rust_ptr_to_jstring(&mut env, result_ptr)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Java_expo_modules_rustysearch_ExpoRustySearchModule_clearIndex(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let result_ptr = clear_index();
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
    pub unsafe extern "C" fn Java_expo_modules_rustysearch_ExpoRustySearchModule_addDocumentsBulk(
        mut env: JNIEnv,
        _class: JClass,
        json: JString,
    ) -> jstring {
        let r_json: String = env.get_string(&json).unwrap().into();
        let c_json = CString::new(r_json).unwrap();
        
        let result_ptr = add_documents_bulk(c_json.as_ptr());
        rust_ptr_to_jstring(&mut env, result_ptr)
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn Java_expo_modules_rustysearch_ExpoRustySearchModule_searchDocuments(
        mut env: JNIEnv,
        _class: JClass,
        query: JString,
    ) -> jstring {
        let r_query: String = env.get_string(&query).unwrap().into();
        let c_query = CString::new(r_query).unwrap();
        
        let result_ptr = search_documents(c_query.as_ptr());
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