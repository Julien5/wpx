import 'package:flutter/material.dart';

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
            child: Text(
              text,
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 12),
            ),
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
    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: Text(
        text,
        textAlign: TextAlign.left,
        style: TextStyle(fontSize: 12),
      ),
    );
  }
}
