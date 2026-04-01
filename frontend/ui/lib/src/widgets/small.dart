import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/screen_configuration.dart';

class SmallButton extends StatelessWidget {
  final VoidCallback? callback;
  final String text;
  const SmallButton({super.key, this.callback, required this.text});

  @override
  Widget build(BuildContext context) {
    EdgeInsets valuePadding = const EdgeInsets.fromLTRB(15, 0, 15, 0);
    return Padding(
      padding: const EdgeInsets.all(8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.start,
        children: [
          OutlinedButton(
            onPressed: callback,
            style: ElevatedButton.styleFrom(
              padding: valuePadding,
              minimumSize: Size.zero,
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
            child: Text(text, textAlign: TextAlign.center),
          ),
        ],
      ),
    );
  }
}

class SmallText extends StatelessWidget {
  final String text;
  const SmallText({super.key, required this.text});

  @override
  Widget build(BuildContext context) {
    context.watch<ScreenConfiguration>();
    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: Text(text, textAlign: TextAlign.left),
    );
  }
}

class SmallCentralWidget extends StatelessWidget {
  final Widget child;
  const SmallCentralWidget({super.key, required this.child});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: 500),
        child: Padding(
          padding: const EdgeInsets.all(20), // Add padding inside the card
          child: child,
        ),
      ),
    );
  }
}
