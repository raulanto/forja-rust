use clap::Args;

#[derive(Args, Debug)]
pub struct DjangoArgs {
    /// Nombre del proyecto a crear
    pub name: String,

    /// Versión de Django a instalar
    #[arg(long = "django-version", default_value = "6.1.1")]
    pub django_version: String,

    /// Nombre de la aplicación inicial
    #[arg(long = "app", default_value = "core")]
    pub app: String,

    /// Levantar runserver al finalizar
    #[arg(long = "run")]
    pub run: bool,
}
