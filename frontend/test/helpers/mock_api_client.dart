import 'package:dio/dio.dart';
import 'package:ethnomusicology_frontend/services/api_client.dart';

/// Shared mock interceptor for API client tests.
/// Captures requests and returns configurable responses.
class MockInterceptor extends Interceptor {
  RequestOptions? lastRequest;
  Object? responseOverride;
  DioException? errorOverride;

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    lastRequest = options;
    if (errorOverride != null) {
      handler.reject(errorOverride!);
      return;
    }
    handler.resolve(Response(
      requestOptions: options,
      statusCode: 200,
      data: responseOverride ?? <String, dynamic>{},
    ));
  }
}

/// Create a test ApiClient with the mock interceptor installed.
({ApiClient client, MockInterceptor interceptor}) createMockApiClient() {
  final dio = Dio(BaseOptions(baseUrl: 'http://localhost:3001/api'));
  final interceptor = MockInterceptor();
  dio.interceptors.add(interceptor);
  return (client: ApiClient(dio: dio), interceptor: interceptor);
}
