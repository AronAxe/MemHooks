"""Native Hermes registration. Importing/registering never contacts a backend."""
from .recall_runtime import RecallRuntime


def profile_home():
    from hermes_constants import get_hermes_home
    return str(get_hermes_home())


def scoped_secret(name):
    from agent.secret_scope import get_secret
    return get_secret(name, "")


def register(ctx):
    runtime = RecallRuntime(lambda: ctx.get_config("runtime", {}), profile_home, scoped_secret)
    ctx.register_hook("pre_llm_call", runtime.pre_llm_call)
    ctx.register_hook("post_tool_call", runtime.post_tool_call)
    ctx.register_hook("on_session_finalize", runtime.clear_session)
    ctx.register_hook("on_session_reset", runtime.clear_session)
