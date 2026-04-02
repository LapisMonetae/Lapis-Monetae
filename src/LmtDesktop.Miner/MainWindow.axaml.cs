using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using LmtDesktop.Miner.ViewModels;
using System.Linq;

namespace LmtDesktop.Miner;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
        DataContext = new MinerMainViewModel();
    }

    public async void BrowseBridgePath_Click(object? sender, RoutedEventArgs e)
    {
        var vm = DataContext as MinerMainViewModel;
        if (vm == null) return;

        var files = await StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
        {
            Title = "Select lmt-stratum-bridge binary",
            AllowMultiple = false,
            FileTypeFilter = new[]
            {
                new FilePickerFileType("Executable") { Patterns = new[] { "*.exe", "*" } },
                new FilePickerFileType("All files") { Patterns = new[] { "*" } },
            }
        });

        if (files.Any())
            vm.BridgePath = files[0].Path.LocalPath;
    }

    public async void BrowseXmrigPath_Click(object? sender, RoutedEventArgs e)
    {
        var vm = DataContext as MinerMainViewModel;
        if (vm == null) return;

        var files = await StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
        {
            Title = "Select XMRig binary",
            AllowMultiple = false,
            FileTypeFilter = new[]
            {
                new FilePickerFileType("Executable") { Patterns = new[] { "*.exe", "*" } },
                new FilePickerFileType("All files") { Patterns = new[] { "*" } },
            }
        });

        if (files.Any())
            vm.XmrigPath = files[0].Path.LocalPath;
    }
}
