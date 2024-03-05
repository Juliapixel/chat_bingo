using bingo_frontend_wasm;
using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using System.Reflection;


class Program
{
    //Globally accessible vars
    internal static string VERSIONNAME { get; set; } = "ms1-proto";
    internal static string? GITHASH { get; set; } = string.Empty;
    internal static string? COMMITDATE { get; set; } = string.Empty;

    internal static ENVIRONMENT_FLAG ENVIRONMENT { get; set; }

    //TODO: Add dev environment flag string

    static async Task Main(string[] args)
    {
        GITHASH = Assembly.GetEntryAssembly()?
                          .GetCustomAttributes<AssemblyMetadataAttribute>()
                          .FirstOrDefault(attr => attr.Key == "GitHash")?.Value;

        COMMITDATE = Assembly.GetEntryAssembly()?
                          .GetCustomAttributes<AssemblyMetadataAttribute>()
                          .FirstOrDefault(attr => attr.Key == "CommitDate")?.Value;


        var builder = WebAssemblyHostBuilder.CreateDefault(args);
        builder.RootComponents.Add<App>("#app");
        builder.RootComponents.Add<HeadOutlet>("head::after");

        builder.Services.AddScoped(sp => new HttpClient { BaseAddress = new Uri(builder.HostEnvironment.BaseAddress) });

        if (builder.HostEnvironment.IsDevelopment())
        {
            ENVIRONMENT = ENVIRONMENT_FLAG.DEVELOPMENT;
            Console.WriteLine($"Development version {VERSIONNAME} {GITHASH} built {COMMITDATE}");
        }
        else
        {
            ENVIRONMENT = ENVIRONMENT_FLAG.PRODUCTION;
            Console.WriteLine($"Production version {VERSIONNAME} {GITHASH} built {COMMITDATE}");
        }

        await builder.Build().RunAsync();
    }
}

enum ENVIRONMENT_FLAG
{
    DEVELOPMENT,
    PRODUCTION
}