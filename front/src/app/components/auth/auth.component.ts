import { Component } from '@angular/core';
import { FormBuilder, FormGroup, Validators, ReactiveFormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatTabsModule } from '@angular/material/tabs';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { CommonModule } from '@angular/common';
import { AuthService } from '../../services/auth.service';


@Component({
  selector: 'app-auth',
  standalone: true, // très important pour Angular 17+
  imports: [
    ReactiveFormsModule,
    MatCardModule,
    MatTabsModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    CommonModule
  ],
  templateUrl: './auth.component.html',
  styleUrls: ['./auth.component.scss'],
})
export class AuthComponent {
  loginForm: FormGroup;
  registerForm: FormGroup;

  constructor(private fb: FormBuilder,private authService: AuthService) {
    this.loginForm = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      password: ['', Validators.required],
    });

    this.registerForm = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      password: ['', Validators.required],
      confirmPassword: ['', Validators.required],
    });
  }

  login() {
    if (this.loginForm.valid) {
      const { email, password } = this.loginForm.value; // destructuring, plus clean
      console.log('Login', email, password);
  
      this.authService.login(email, password).subscribe({
        next: (res) => console.log('Login success', res),
        error: (err) => console.error('Login error', err),
      });
    }
  }

  register() {
    if (this.registerForm.valid) {
      const { email, password, confirmPassword } = this.registerForm.value;
  
      if (password !== confirmPassword) {
        console.error('Passwords do not match');
        return;
      }
  
      this.authService.register(email, password).subscribe({
        next: (res) => console.log('Register success', res),
        error: (err) => console.error('Register error', err),
      });
    }
  }
}
