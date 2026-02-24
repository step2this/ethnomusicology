import 'package:dio/dio.dart';

class ApiClient {
  static const _baseUrl = 'http://localhost:3001/api';

  final Dio _dio;

  ApiClient({Dio? dio})
      : _dio = dio ??
            Dio(BaseOptions(
              baseUrl: _baseUrl,
              connectTimeout: const Duration(seconds: 10),
              receiveTimeout: const Duration(seconds: 10),
            ));

  Dio get dio => _dio;
}
