import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/rust/api/frontend.dart';

class ProfileWidget extends StatefulWidget {
  const ProfileWidget({super.key});

  @override
  State<ProfileWidget> createState() => ProfileWidgetState();
}

class ProfileWidgetState extends State<ProfileWidget> {
  String? svgData;
  String? errorMessage;
  bool isLoading = true;

  @override
  void initState() {
    super.initState();
    loadCircle();
  }

  Future<void> loadCircle() async {
    setState(() {
      isLoading = true;
      svgData = null; // Clear previous SVG data
      errorMessage = null; // Clear previous error message
    });
    try {
      final data = await svgCircle();
      setState(() {
        svgData = data;
        isLoading = false;
      });
    } catch (e) {
      setState(() {
        errorMessage = 'Error: $e';
        isLoading = false;
      });
    }
  }

  Future<void> loadProfile(Frontend frontend) async {
    setState(() {
      isLoading = true;
      svgData = null; // Clear previous SVG data
      errorMessage = null; // Clear previous error message
    });
    try {
      //final data = await svgCircle();
      final data = await frontend.svg();
      setState(() {
        svgData = data;
        isLoading = false;
      });
    } catch (e) {
      setState(() {
        errorMessage = 'Error: $e';
        isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 500.0, // Fixed width
      height: 250.0, // Fixed height
      child: Builder(
        builder: (context) {
          if (isLoading) {
            return const Center(
              child: CircularProgressIndicator(), // CircularProgressIndicator without extra SizedBox
            );
          } else if (errorMessage != null) {
            return Center(child: Text(errorMessage!));
          } else if (svgData != null) {
            return SvgPicture.string(
              svgData!,
              width: 500,
              height: 250,
            );
          } else {
            return const Center(child: Text('No data available'));
          }
        },
      ),
    );
  }
}
