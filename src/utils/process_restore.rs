use crate::structures::elements::Elements;
use crate::structures::settings::Settings;
use crate::utils::s3_utils::get_file_from_s3;
use log::error;
use s3::Bucket;

/// Restores specified elements from an S3 bucket to the local system asynchronously.
///
/// This function retrieves each element's backup file from the S3 bucket using the provided
/// `restore_dir` and `s3_folder` of each element. After downloading the file, it attempts to restore
/// the element using the `perform_restore` method. If any error occurs during downloading or restoring,
/// it logs the error and moves to the next element.
///
/// # Arguments
/// - `bucket` - The S3 bucket from which the backup files will be retrieved.
/// - `restore_dir` - The directory within the S3 bucket that contains the backup files to be restored.
/// - `elements` - A slice of references to the elements that need to be restored.
///
/// # Returns
/// This function does not return a value. It performs the restoration operation for each element,
/// logging errors if any occur during the process.
///
/// # Errors
/// This function will log errors if:
/// - The file for an element cannot be retrieved from S3.
/// - The restoration operation for an element fails.
///
/// # Example
/// ```rust
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// let restore_dir = "path/to/restore".to_string();
/// let elements: Vec<&Elements> = vec![/* elements to restore */];
/// restore_elements(&bucket, &restore_dir, &elements).await;
/// ```
async fn restore_elements(bucket: &Bucket, restore_dir: &String, elements: &[&Elements]) {
    for element in elements {
        let file_path = match get_file_from_s3(bucket, restore_dir, &element.s3_folder).await {
            Ok(file) => file,
            Err(e) => {
                error!("{}", e.to_string());
                continue;
            }
        };

        if let Err(e) = element.perform_restore(&file_path).await {
            error!("{}", e.to_string());
        }
    }
}

/// Initiates the restoration process for all elements from the S3 bucket.
///
/// This function constructs the restore directory path from the settings and attempts to restore
/// all elements listed in the `settings.elements` vector. It calls the `restore_elements` function
/// to perform the actual restoration.
///
/// # Arguments
/// - `settings` - The configuration settings containing the elements to be restored.
/// - `bucket` - The S3 bucket from which the backup files will be restored.
///
/// # Returns
/// This function does not return a value. It performs the restoration process for all elements listed
/// in the `settings` configuration.
///
/// # Example
/// ```rust
/// let settings: Settings = /* Obtain settings from configuration */;
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// restore_all_process(&settings, &bucket).await;
/// ```
pub async fn restore_all_process(settings: &Settings, bucket: &Bucket) {
    let restore_dir = format!("{}/to_restore", &settings.backup_dir);

    restore_elements(
        bucket,
        &restore_dir,
        &settings.elements.iter().collect::<Vec<_>>(),
    )
    .await;
}

/// Initiates the restoration process for selected elements from the S3 bucket based on provided arguments.
///
/// This function constructs the restore directory path from the settings and filters the elements to restore
/// based on the arguments passed to it. Only the elements whose `element_title` matches the arguments will
/// be restored. If no matching elements are found, it logs an error. The function uses `restore_elements`
/// to perform the restoration.
///
/// # Arguments
/// - `settings` - The configuration settings containing the elements to be restored.
/// - `bucket` - The S3 bucket from which the selected backup files will be restored.
/// - `args` - A vector of strings representing the arguments passed to the function, used to filter the elements.
///
/// # Returns
/// This function does not return a value. It performs the restoration process for the selected elements based
/// on the filtered arguments.
///
/// # Errors
/// This function will log an error if no matching elements are found for the provided arguments.
///
/// # Example
/// ```rust
/// let settings: Settings = /* Obtain settings from configuration */;
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// let args = vec!["restore", "element1", "element2"];
/// restore_selected_process(&settings, &bucket, &args).await;
/// ```
pub async fn restore_selected_process(settings: &Settings, bucket: &Bucket, args: &Vec<String>) {
    let restore_dir = format!("{}/to_restore", &settings.backup_dir);

    let filtered_args: Vec<_> = args.iter().skip(2).collect();

    let selected_elements: Vec<_> = settings
        .elements
        .iter()
        .filter(|element| filtered_args.contains(&&element.element_title))
        .collect();

    if selected_elements.is_empty() {
        error!(
            "No matching elements found for the provided arguments: {:?}",
            args
        );
        return;
    }

    restore_elements(bucket, &restore_dir, &selected_elements).await;
}
