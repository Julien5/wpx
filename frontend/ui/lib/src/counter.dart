import 'dart:developer' as developer;

import 'package:flutter/material.dart';

class Counter extends StatelessWidget {
  final VoidCallback? onCounterPressed;

  const Counter({super.key, required this.onCounterPressed});

  @override
  Widget build(BuildContext context) {
    return ElevatedButton(
      onPressed: onCounterPressed ?? () {
        developer.log('Counter pressed', name: 'Counter');
      },
      style: ElevatedButton.styleFrom(
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(4.0), // Smaller corner radius
        ),
      ),
      child: const Text('Press me'),
    );
  }
}