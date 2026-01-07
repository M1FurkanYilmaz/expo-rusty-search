#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Initialize or load the index from disk
 * Parameters:
 *   - index_path_ptr: C string pointer to the directory path where index should be stored
 * Returns: Success or error message
 */
const char *initialize_index(const char *index_path_ptr);

/**
 * Function to add a document to the index
 * Parameters:
 *   - title_ptr: C string pointer to the document title
 *   - body_ptr: C string pointer to the document body
 * Returns: Success or error message
 */
const char *add_document(const char *title_ptr, const char *body_ptr);

/**
 * Function to search through all added documents
 * Parameters:
 *   - query_ptr: C string pointer to the search query
 * Returns: Search results or error message
 */
const char *search_basic(const char *query_ptr);

/**
 * Get the number of documents in the index
 */
const char *get_document_count(void);

void free_string(char *ptr);
