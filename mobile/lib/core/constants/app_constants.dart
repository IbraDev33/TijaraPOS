/// App-wide constants. Networking/API constants live here rather than
/// scattered through feature modules so the pairing/API base path only
/// needs to change in one place.
abstract final class AppConstants {
  static const String appName = 'TijaraPOS';

  /// Matches the Desktop API versioning scheme (`/api/v1/...`).
  static const String apiVersionPrefix = '/api/v1';

  static const int defaultServerPort = 8080;
}
