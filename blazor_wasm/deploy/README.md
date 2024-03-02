#### Blazor Frontend Deployment Kit
Intended to work with an instance that has NGINX installed

To use:
- Run the "Publish" function on the project in Visual Studio, or run 'dotnet build'
- Copy the contents of the "wwwroot" folder from the output directory into the "webstaging" folder
- Copy all files to the home directory of the instance
- Run 'bash DeployBlazor.sh'