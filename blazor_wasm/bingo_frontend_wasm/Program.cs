using bingo_frontend_wasm;
using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using System.Reflection;


class Program
{
    internal static string GITHASH { get; set; } = "";

    static async Task Main(string[] args)
    {
        GITHASH = Assembly.GetEntryAssembly()
                          .GetCustomAttributes<AssemblyMetadataAttribute>()
                          .FirstOrDefault(attr => attr.Key == "GitHash")?.Value;

        Console.WriteLine(GITHASH);

        var builder = WebAssemblyHostBuilder.CreateDefault(args);
        builder.RootComponents.Add<App>("#app");
        builder.RootComponents.Add<HeadOutlet>("head::after");

        builder.Services.AddScoped(sp => new HttpClient { BaseAddress = new Uri(builder.HostEnvironment.BaseAddress) });


        await builder.Build().RunAsync();
    }
}