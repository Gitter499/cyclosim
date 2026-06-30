import AppKit
import Foundation
import VeloFFI

/// Thin wrapper over the Rust ride library (SQLite + on-disk artifacts).
public enum LocalRideStore {
  public static func defaultLibrary() throws -> RideLibraryHandle {
    try RideLibraryHandle.withDefaults()
  }

  public static func open(library: RideLibraryHandle) -> LocalRideStoreHandle {
    LocalRideStoreHandle(library: library)
  }

  public static var ridesDirectory: URL {
    let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
    return docs.appendingPathComponent("VeloSim/rides", isDirectory: true)
  }
}

public final class LocalRideStoreHandle {
  private let library: RideLibraryHandle

  public init(library: RideLibraryHandle) {
    self.library = library
  }

  public func listRides() throws -> [RideRecordDto] {
    try library.listRides()
  }

  public func getRide(id: String) throws -> RideRecordDto? {
    try library.getRide(id: id)
  }

  public func deleteRide(id: String) throws -> Bool {
    try library.deleteRide(id: id)
  }

  public func rideFolder(for record: RideRecordDto) -> URL {
    URL(fileURLWithPath: record.fitPath).deletingLastPathComponent()
  }

  public func openInFinder(_ record: RideRecordDto) {
    let folder = rideFolder(for: record)
    NSWorkspace.shared.open(folder)
  }

  public func stravaURL(for record: RideRecordDto) -> URL? {
    guard let id = record.stravaActivityId else { return nil }
    return URL(string: "https://www.strava.com/activities/\(id)")
  }
}
