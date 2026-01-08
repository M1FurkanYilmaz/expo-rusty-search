#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Initialize a new in-memory index
 * Returns: Success or error message
 */
const char *initialize_index(void);

/**
 * Clear the index and reinitialize it
 * This is useful when you want to rebuild the index from scratch
 */
const char *clear_index(void);

/**
 * Add a single document to the index
 * Parameters:
 *   - title_ptr: C string pointer to the document title
 *   - body_ptr: C string pointer to the document body
 * Returns: Success or error message
 */
const char *add_document(const char *title_ptr, const char *body_ptr);

/**
 * Add multiple documents at once from a JSON array
 * Parameters:
 *   - json_ptr: C string pointer to JSON array like [{"title":"..","body":".."}, ...]
 * Returns: Success message with count or error message
 */
const char *add_documents_bulk(const char *json_ptr);

/**
 * Search through all indexed documents
 * Parameters:
 *   - query_ptr: C string pointer to the search query
 * Returns: JSON array of search results
 */
const char *search_documents(const char *query_ptr);

/**
 * Get the number of documents in the index
 */
const char *get_document_count(void);

void free_string(char *ptr);
