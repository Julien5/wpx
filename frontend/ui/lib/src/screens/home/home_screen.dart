import 'dart:developer' as developer;
import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/main.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/screens/home/trackfile_list_widget.dart';

class _ChooseData extends StatefulWidget {
  const _ChooseData();

  @override
  State<_ChooseData> createState() => _ChooseDataState();
}

class _ChooseDataState extends State<_ChooseData> {
  PendingContent? findResult;
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
    try {
      for (var file in result.files) {
        if (file.bytes == null) {
          developer.log("read: ${file.path}");
          bytes.add(await File(file.path!).readAsBytes());
        } else {
          bytes.add(file.bytes!.buffer.asInt8List().toList());
        }
      }
      onDone(PendingContent.makeFromBytes(bytes));
    } on Exception catch (_, e) {
      debugPrint(e.toString());
    }
  }

  void chooseDemo() {
    List<int> bytes = bridge.demoBytes();
    onDone(PendingContent.makeFromBytes([bytes]));
  }

  void onDone(PendingContent userInput) async {
    RootModel root = context.read<RootModel>();
    root.setPendingContent(userInput);
    gotoLoad(context);
  }

  @override
  Widget build(BuildContext ctx) {
    return SafeArea(
      child: Stack(
        children: [
          Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.start,
              children: [
                // the image seems blurry, even so it shown at its native size.
                Flexible(child: Image.asset('assets/images/png/home.png')),
                const SizedBox(height: 20),
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    ElevatedButton(
                      onPressed: loading ? null : () => chooseGPX(),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: Colors.white,
                        side: const BorderSide(
                          color: Colors.blueAccent,
                          width: 1,
                        ),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(5),
                        ),
                      ),
                      child: const Text("Open GPX files"),
                    ),
                    if (errorMessage != null)
                      Padding(
                        padding: const EdgeInsets.only(top: 10),
                        child: Text(
                          errorMessage!,
                          style: const TextStyle(color: Colors.red),
                        ),
                      ),
                    const SizedBox(width: 20),
                  ],
                ),
                const SizedBox(height: 20),
                Expanded(child: TrackFileListWidget()),
              ],
            ),
          ),
          Positioned(
            bottom: 16,
            right: 16,
            child: ElevatedButton(
              onPressed: loading ? null : () => chooseDemo(),
              style: ElevatedButton.styleFrom(
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(5),
                ),
              ),
              child: const Text("Load sample", style: TextStyle(fontSize: 13)),
            ),
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
    PackageModel info = ctx.watch<PackageModel>();
    return Scaffold(
      appBar: AppBar(title: Text('WPX ${info.packageInfo.version}')),
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
