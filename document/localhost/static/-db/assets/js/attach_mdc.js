window.addEventListener('DOMContentLoaded', (event) => {
    document.querySelectorAll('.mdc-text-field').forEach(function(e){
        mdc.textField.MDCTextField.attachTo(e);
    });
    document.querySelectorAll('.mdc-button').forEach(function(e){
        mdc.ripple.MDCRipple.attachTo(e);
    });
});