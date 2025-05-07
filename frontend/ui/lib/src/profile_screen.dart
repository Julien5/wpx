import 'package:flutter/material.dart';
import 'package:ui/src/profile_widget.dart';
import 'package:ui/src/counter.dart';
import 'package:ui/src/rust/api/frontend.dart';
import 'dart:developer' as developer;

class ProfileScreen extends StatefulWidget {
  const ProfileScreen({super.key});

  @override
  State<ProfileScreen> createState() => _ProfileScreenState();
}

class _ProfileScreenState extends State<ProfileScreen> {
  final GlobalKey<ProfileWidgetState> _profileKey =
      GlobalKey<ProfileWidgetState>();
  Frontend? frontend;

  @override
  void initState() {
    super.initState();
    _initializeFrontend();
  }

  Future<void> _initializeFrontend() async {
    final instance = await Frontend.create();
    setState(() {
      frontend = instance;
    });
  }

  void _reloadProfile() {
    if (frontend == null) {
      developer.log(name:'screen',"frontend is null");
      return;
    }
    developer.log(name:'screen',"frontend is not null");
    Frontend f=frontend!;
    f.changeParameter(eps: 10.0);
    _profileKey.currentState?.loadProfile(f);
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16.0), // Add padding around ProfileWidget
          decoration: BoxDecoration(
            border: Border.all(color: Colors.blue, width: 2.0), // Add a blue border
            borderRadius: BorderRadius.circular(8.0), // Optional: Rounded corners
          ),
          child: ProfileWidget(key: _profileKey), // ProfileWidget inside the container
        ),
        const SizedBox(height: 60),
        Counter(onCounterPressed: _reloadProfile),
      ],
    );
  }
}
