import 'dart:developer' as developer;
import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:provider/provider.dart';
import 'package:ui/main.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/routes.dart';

class _ChooseData extends StatefulWidget {
  const _ChooseData();

  @override
  State<_ChooseData> createState() => _ChooseDataState();
}

class _ChooseDataState extends State<_ChooseData> {
  UserInput? findResult;
  String? errorMessage;
  bool loading = false;

  void chooseGPX() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ["gpx"],
    );
    if (result == null) {
      return;
    }
    if (!mounted) {
      return;
    }
    developer.log("result: ${result.count}");
    for (var file in result.files) {
      List<int> bytes = [];
      if (file.bytes == null) {
        bytes = await File(file.path!).readAsBytes();
      } else {
        bytes = file.bytes!.buffer.asInt8List().toList();
      }
      onDone(UserInput.makeFromBytes(bytes));
      break;
    }
  }

  void chooseDemo() {
    onDone(UserInput.makeDemo());
  }

  void gotoLoad(BuildContext ctx) {
    final location = GoRouterState.of(context).matchedLocation;
    debugPrint('Current location: $location');
    ctx.go(Routes.load);
  }

  void onDone(UserInput userInput) async {
    RootModel root = Provider.of<RootModel>(context, listen: false);
    root.setUserInput(userInput);
    gotoLoad(context);
  }

  @override
  Widget build(BuildContext ctx) {
    return Center(
      child: Column(
        children: [
          SizedBox(height: 40),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Image.asset(
                'assets/images/png/combined.png',
                width: 250,
                fit: BoxFit.cover,
              ),
            ],
          ),
          SizedBox(height: 40),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              ElevatedButton(
                onPressed: loading ? null : () => chooseGPX(),
                child: const Text("GPX file"),
              ),
              if (errorMessage !=
                  null) // Conditionally display the error message
                Padding(
                  padding: const EdgeInsets.only(top: 10),
                  child: Text(
                    errorMessage!,
                    style: const TextStyle(color: Colors.red),
                  ),
                ),
              const SizedBox(width: 20),
              ElevatedButton(
                onPressed: loading ? null : () => chooseDemo(),
                child: const Text("Demo"),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _HomeBody extends StatelessWidget {
  @override
  Widget build(BuildContext ctx) {
    return _ChooseData();
  }
}

class HomeScaffold extends StatelessWidget {
  const HomeScaffold({super.key});

  @override
  Widget build(BuildContext ctx) {
    PackageInfo info = Provider.of<PackageModel>(ctx).packageInfo;
    return Scaffold(
      appBar: AppBar(title: Text('WPX ${info.version}')),
      body: _HomeBody(),
    );
  }
}

/*class HomeProviders extends MultiProvider {
  final Widget child;
  HomeProviders({super.key, required this.child})
    : super(
        providers: [
          ChangeNotifierProvider(
            create: (_) => HomeModel(packageInfo: packageInfo),
          ),
        ],
        child: child,
      );
}*/

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return HomeScaffold();
  }
}
