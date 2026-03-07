class PurchaseLink {
  final String store;
  final String name;
  final String url;
  final String icon;

  PurchaseLink({
    required this.store,
    required this.name,
    required this.url,
    required this.icon,
  });

  factory PurchaseLink.fromJson(Map<String, dynamic> json) => PurchaseLink(
        store: json['store'] as String,
        name: json['name'] as String,
        url: json['url'] as String,
        icon: json['icon'] as String,
      );
}
