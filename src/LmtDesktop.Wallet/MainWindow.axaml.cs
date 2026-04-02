using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using LmtDesktop.Wallet.ViewModels;
using System.Linq;

namespace LmtDesktop.Wallet;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
        DataContext = new MainWindowViewModel();
    }

    public async void BrowseCliPath_Click(object? sender, RoutedEventArgs e)
    {
        var vm = DataContext as MainWindowViewModel;
        if (vm == null) return;

        var files = await StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
        {
            Title = "Select lmt-cli binary",
            AllowMultiple = false,
            FileTypeFilter = new[]
            {
                new FilePickerFileType("Executable") { Patterns = new[] { "*.exe", "*" } },
                new FilePickerFileType("All files") { Patterns = new[] { "*" } },
            }
        });

        if (files.Any())
        {
            var path = files[0].Path.LocalPath;
            vm.CliPath = path;
        }
    }
}
