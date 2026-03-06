import 'dart:developer' as developer;
import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:provider/provider.dart';
import 'package:ui/main.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

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
      allowMultiple: true,
    );
    if (result == null) {
      return;
    }
    if (!mounted) {
      return;
    }

    List<List<int>> bytes = [];
    for (var file in result.files) {
      if (file.bytes == null) {
        developer.log("read: ${file.path}");
        bytes.add(await File(file.path!).readAsBytes());
      } else {
        bytes.add(file.bytes!.buffer.asInt8List().toList());
      }
    }
    onDone(UserInput.makeFromBytes(bytes));
  }

  void chooseDemo() {
    List<int> bytes = bridge.demoBytes();
    onDone(UserInput.makeFromBytes([bytes]));
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
