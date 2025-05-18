import 'package:flutter/material.dart';

class PressButton extends StatelessWidget {
  final VoidCallback? onCounterPressed;
  final String label;

  const PressButton({
    super.key,
    required this.label,
    required this.onCounterPressed,
  });

  @override
  Widget build(BuildContext context) {
    return ElevatedButton(
      onPressed: onCounterPressed,
      style: ElevatedButton.styleFrom(
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(4.0), // Smaller corner radius
        ),
      ),
      child: Text(label),
    );
  }
}
