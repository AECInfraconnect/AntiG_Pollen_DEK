import Foundation
import NetworkExtension
import os.log

final class FilterManager {
    static let shared = FilterManager()

    private let logger = Logger(
        subsystem: "com.aecinfraconnect.pollen.dek",
        category: "FilterManager"
    )

    func enableFilter(completion: @escaping (Result<Void, Error>) -> Void) {
        NEFilterManager.shared().loadFromPreferences { [weak self] error in
            if let error = error {
                completion(.failure(error))
                return
            }

            let providerConfig = NEFilterProviderConfiguration()
            providerConfig.username = NSUserName()
            providerConfig.organization = "Pollen DEK"
            providerConfig.filterSockets = true
            providerConfig.filterBrowsers = true

            // Provider-specific configuration. Keep this small.
            providerConfig.vendorConfiguration = [
                "policyMode": "local-cache-first",
                "appGroup": "group.com.aecinfraconnect.pollen.dek",
                "auditMode": "metadata-only"
            ]

            let manager = NEFilterManager.shared()
            manager.localizedDescription = "Pollen DEK Network Filter"
            manager.providerConfiguration = providerConfig
            manager.isEnabled = true

            manager.saveToPreferences { saveError in
                if let saveError = saveError {
                    self?.logger.error("Failed to save NEFilter preferences: \(saveError.localizedDescription)")
                    completion(.failure(saveError))
                } else {
                    self?.logger.info("Pollen DEK content filter enabled")
                    completion(.success(()))
                }
            }
        }
    }

    func disableFilter(completion: @escaping (Result<Void, Error>) -> Void) {
        NEFilterManager.shared().loadFromPreferences { error in
            if let error = error {
                completion(.failure(error))
                return
            }

            let manager = NEFilterManager.shared()
            manager.isEnabled = false
            manager.saveToPreferences { saveError in
                if let saveError = saveError {
                    completion(.failure(saveError))
                } else {
                    completion(.success(()))
                }
            }
        }
    }
}
