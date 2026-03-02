import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../services/api_client.dart';

/// Canonical provider for the API client.
/// All providers that need an [ApiClient] should depend on this.
final apiClientProvider = Provider<ApiClient>((ref) => ApiClient());
